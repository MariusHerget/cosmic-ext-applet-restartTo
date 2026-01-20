// SPDX-License-Identifier: MPL-2.0

use efibootnext::{BootEntry, BootEntryId};
use std::fmt;

/// Represents an EFI boot entry with its ID and description
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootEntryInfo {
    pub id: BootEntryId,
    pub description: String,
    pub active: bool,
}

impl From<BootEntry> for BootEntryInfo {
    fn from(entry: BootEntry) -> Self {
        BootEntryInfo {
            id: entry.id,
            description: entry.description,
            active: entry.active,
        }
    }
}

impl fmt::Display for BootEntryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

/// Get all available EFI boot entries
pub fn get_boot_entries() -> Result<Vec<BootEntryInfo>, String> {
    let entries = efibootnext::list_boot_entries()
        .map_err(|e| format!("Failed to list boot entries: {}", e))?;

    Ok(entries.into_iter().map(BootEntryInfo::from).collect())
}

/// Set the BootNext EFI variable to boot into a specific entry on next reboot
pub fn set_boot_next(entry_id: BootEntryId) -> Result<(), String> {
    efibootnext::set_boot_next(entry_id)
        .map_err(|e| format!("Failed to set BootNext: {}", e))?;
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
