// SPDX-License-Identifier: MPL-2.0

use crate::boot::BootEntryInfo;
use crate::config::Config;
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use futures_util::SinkExt;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Cached boot entries.
    boot_entries: Vec<BootEntryInfo>,
    /// Entry pending confirmation for reboot.
    selected_entry: Option<BootEntryInfo>,
    /// Loading state indicator.
    loading_entries: bool,
    /// Error message to display.
    error_message: Option<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    /// Trigger boot entry discovery.
    LoadBootEntries,
    /// Receive loaded boot entries.
    BootEntriesLoaded(Result<Vec<BootEntryInfo>, String>),
    /// User selected a boot entry.
    SelectBootEntry(BootEntryInfo),
    /// User confirmed reboot dialog.
    ConfirmReboot,
    /// User cancelled reboot dialog.
    CancelReboot,
    /// Handle reboot errors.
    RebootError(String),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.cosmic_ext.restartTo";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => {
                        tracing::debug!("Successfully loaded app configuration");
                        config
                    }
                    Err((errors, config)) => {
                        for why in &errors {
                            tracing::error!(%why, "error loading app config");
                        }
                        tracing::warn!("Using default configuration due to errors");
                        config
                    }
                })
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Failed to create config context, using defaults");
                    Config::default()
                }),
            ..Default::default()
        };

        tracing::info!("Applet initialized, loading boot entries");
        // Load boot entries on initialization
        (app, Task::perform(
            async { Message::LoadBootEntries },
            cosmic::Action::App,
        ))
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("system-reboot-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        // Show confirmation dialog if an entry is selected
        if let Some(ref entry) = self.selected_entry {
            let dialog_content = widget::column()
                .spacing(16)
                .padding(24)
                .push(
                    widget::text({
                        use std::collections::HashMap;
                        let mut args = HashMap::new();
                        args.insert("entry", entry.description.as_str());
                        fl!("confirm-restart", args)
                    })
                        .size(16),
                )
                .push(
                    widget::row()
                        .spacing(8)
                        .push(
                            widget::button::standard(fl!("cancel-button"))
                                .on_press(Message::CancelReboot),
                        )
                        .push(
                            widget::button::suggested(fl!("restart-button"))
                                .on_press(Message::ConfirmReboot),
                        ),
                );
            return self.core.applet.popup_container(dialog_content).into();
        }

        let content_list = widget::list_column()
            .padding(5)
            .spacing(0);

        let content_list = if self.loading_entries {
            content_list.add(
                widget::settings::item(
                    fl!("loading-entries"),
                    widget::text(""),
                ),
            )
        } else if let Some(ref error) = self.error_message {
            // Check if it's a permission error and format it better
            if error.contains("Permission denied") || error.contains("permission denied") {
                // For permission errors, show a more detailed message
                content_list
                    .add(
                        widget::settings::item(
                            fl!("error-permission"),
                            widget::text(fl!("error-permission-details")),
                        ),
                    )
                    .add(
                        widget::column()
                            .spacing(4)
                            .padding([8, 16])
                            .push(widget::text(error).size(10)),
                    )
            } else {
                content_list.add(
                    widget::settings::item(
                        fl!("error-loading"),
                        widget::text(error),
                    ),
                )
            }
        } else if self.boot_entries.is_empty() {
            content_list.add(
                widget::settings::item(
                    fl!("no-boot-entries"),
                    widget::text(""),
                ),
            )
        } else {
            let mut list = content_list;
            for entry in &self.boot_entries {
                let entry_clone = entry.clone();
                list = list.add(
                    widget::button::text(&entry.description)
                        .on_press(Message::SelectBootEntry(entry_clone)),
                );
            }
            list
        };

        self.core.applet.popup_container(content_list).into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    for why in &update.errors {
                        tracing::error!(%why, "app config error");
                    }
                    tracing::debug!("Configuration updated");
                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {
                // For example purposes only.
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::LoadBootEntries => {
                tracing::debug!("Loading boot entries");
                self.loading_entries = true;
                self.error_message = None;
                return Task::perform(
                    async { crate::boot::get_boot_entries() },
                    |result| cosmic::Action::App(Message::BootEntriesLoaded(result)),
                );
            }
            Message::BootEntriesLoaded(result) => {
                self.loading_entries = false;
                match result {
                    Ok(entries) => {
                        tracing::info!(count = entries.len(), "Boot entries loaded successfully");
                        self.boot_entries = entries;
                        self.error_message = None;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load boot entries");
                        self.error_message = Some(e);
                        self.boot_entries.clear();
                    }
                }
            }
            Message::SelectBootEntry(entry) => {
                tracing::info!(
                    boot_entry_id = entry.id,
                    description = %entry.description,
                    "User selected boot entry"
                );
                self.selected_entry = Some(entry);
            }
            Message::ConfirmReboot => {
                if let Some(ref entry) = self.selected_entry {
                    let entry_id = entry.id;
                    tracing::info!(
                        boot_entry_id = entry_id,
                        description = %entry.description,
                        "User confirmed reboot to boot entry"
                    );
                    // Set BootNext and reboot (both are async now)
                    return Task::perform(
                        async move {
                            // First set BootNext with polkit authorization
                            match crate::boot::set_boot_next(entry_id).await {
                                Ok(()) => {
                                    tracing::info!("BootNext set successfully, initiating reboot");
                                    // Then reboot
                                    crate::boot::reboot_system().await
                                }
                                Err(e) => Err(e),
                            }
                        },
                        |result| match result {
                            Ok(()) => {
                                tracing::info!("Reboot initiated successfully");
                                cosmic::Action::App(Message::TogglePopup)
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to set BootNext or reboot");
                                cosmic::Action::App(Message::RebootError(
                                    format!("{}: {}", fl!("error-reboot"), e),
                                ))
                            }
                        },
                    );
                } else {
                    tracing::warn!("ConfirmReboot received but no entry selected");
                }
            }
            Message::CancelReboot => {
                tracing::debug!("User cancelled reboot");
                self.selected_entry = None;
            }
            Message::RebootError(error) => {
                tracing::error!(error = %error, "Reboot error occurred");
                self.error_message = Some(error);
                self.selected_entry = None;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    // Close popup and clear selection
                    tracing::debug!("Closing applet popup");
                    self.selected_entry = None;
                    destroy_popup(p)
                } else {
                    // Open popup and load boot entries if not already loaded
                    tracing::debug!("Opening applet popup");
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    // Load boot entries when opening popup
                    if self.boot_entries.is_empty() && !self.loading_entries {
                        tracing::debug!("Boot entries empty, loading on popup open");
                        return Task::batch(vec![
                            get_popup(popup_settings),
                            Task::perform(
                                async { Message::LoadBootEntries },
                                cosmic::Action::App,
                            ),
                        ]);
                    }
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    tracing::debug!("Popup closed by user");
                    self.popup = None;
                    self.selected_entry = None;
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
