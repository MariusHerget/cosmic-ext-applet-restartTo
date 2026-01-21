// SPDX-License-Identifier: MPL-2.0

use efibootnext::Adapter;
use std::fmt;

/// Represents an EFI boot entry with its ID and description
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootEntryInfo {
    pub id: u16,
    pub description: String,
    pub active: bool,
}

impl fmt::Display for BootEntryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

/// Get all available EFI boot entries
pub fn get_boot_entries() -> Result<Vec<BootEntryInfo>, String> {
    let mut adapter = Adapter::default();
    let entries = adapter
        .load_options()
        .map_err(|e| format!("Failed to list boot entries: {}", e))?;

    Ok(entries
        .into_iter()
        .filter_map(|entry_result| {
            entry_result
                .map(|entry| BootEntryInfo {
                    id: entry.number,
                    description: entry.description,
                    active: true, // LoadOption doesn't expose active status
                })
                .ok()
        })
        .collect())
}

/// Set the BootNext EFI variable to boot into a specific entry on next reboot
pub fn set_boot_next(entry_id: u16) -> Result<(), String> {
    let mut adapter = Adapter::default();
    adapter
        .set_boot_next(entry_id)
        .map_err(|e| {
            let error_msg = format!("{}", e);
            // Check if it's a permission error
            if error_msg.contains("permission denied") || error_msg.contains("Permission denied") {
                format!(
                    "Permission denied: EFI variable access requires elevated privileges.\n\n\
                    Solutions:\n\
                    1. Configure polkit policy (see README.md)\n\
                    2. Run with sudo (not recommended)\n\
                    3. Add user to appropriate group if configured\n\n\
                    Original error: {}",
                    error_msg
                )
            } else {
                format!("Failed to set BootNext: {}", error_msg)
            }
        })?;
    Ok(())
}

/// Reboot the system using systemd-logind via D-Bus
pub async fn reboot_system() -> Result<(), String> {
    use zbus::Connection;

    let connection = Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system D-Bus: {}", e))?;

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await
    .map_err(|e| format!("Failed to create login1 proxy: {}", e))?;

    proxy
        .call_method("Reboot", &(true))
        .await
        .map_err(|e| format!("Failed to call Reboot method: {}", e))?;
    // Note: call_method returns a Message, but we don't need to use it

    Ok(())
}
