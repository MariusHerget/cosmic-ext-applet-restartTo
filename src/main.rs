// SPDX-License-Identifier: MPL-2.0

mod app;
mod boot;
mod config;
mod i18n;

fn main() -> cosmic::iced::Result {
    // Initialize tracing subscriber for logging to journalctl
    // This will log to systemd journal when running as a systemd service,
    // or to stderr when running manually (which journalctl can capture)
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr) // stderr is captured by systemd/journalctl
        .init();

    tracing::info!("Starting cosmic-ext-applet-restart-to");

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    tracing::debug!("Localization initialized for languages: {:?}", requested_languages);

    // Starts the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<app::AppModel>(())
}
