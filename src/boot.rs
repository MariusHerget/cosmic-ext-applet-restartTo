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
    tracing::debug!("Loading EFI boot entries");
    let mut adapter = Adapter::default();
    let entries = adapter
        .load_options()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list boot entries");
            format!("Failed to list boot entries: {}", e)
        })?;

    let boot_entries: Vec<BootEntryInfo> = entries
        .into_iter()
        .filter_map(|entry_result| {
            entry_result
                .map(|entry| {
                    tracing::debug!(id = entry.number, description = %entry.description, "Found boot entry");
                    BootEntryInfo {
                        id: entry.number,
                        description: entry.description,
                        active: true, // LoadOption doesn't expose active status
                    }
                })
                .map_err(|e| {
                    tracing::warn!(error = %e, "Failed to parse boot entry, skipping");
                    e
                })
                .ok()
        })
        .collect();

    tracing::info!(count = boot_entries.len(), "Loaded boot entries successfully");
    Ok(boot_entries)
}

/// Set the BootNext EFI variable using pkexec and efibootmgr
/// This uses pkexec to run efibootmgr with elevated privileges after polkit authorization
async fn set_boot_next_via_pkexec(entry_id: u16) -> Result<(), String> {
    use tokio::process::Command;
    
    tracing::debug!(boot_entry_id = entry_id, "Setting BootNext via pkexec/efibootmgr");
    
    // Format entry_id as 4-digit hex (e.g., 0003)
    let entry_id_hex = format!("{:04X}", entry_id);
    
    // Use pkexec to run efibootmgr with elevated privileges
    // pkexec will prompt for authentication using polkit
    let output = Command::new("pkexec")
        .arg("efibootmgr")
        .arg("-n")
        .arg(&entry_id_hex)
        .output()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to execute pkexec");
            format!("Failed to execute pkexec: {}", e)
        })?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            boot_entry_id = entry_id,
            exit_code = ?output.status.code(),
            stderr = %stderr,
            stdout = %stdout,
            "pkexec/efibootmgr failed"
        );
        
        // Check if it's a permission/cancellation error
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err(format!(
                "Authentication cancelled or efibootmgr not found. Please ensure:\n\
                1. Polkit policy is installed: /usr/share/polkit-1/actions/com.github.cosmic_ext.restartTo.policy\n\
                2. efibootmgr is installed: sudo apt install efibootmgr\n\
                3. You authenticate when prompted"
            ));
        }
        
        return Err(format!(
            "Failed to set BootNext: efibootmgr exited with code {}. Error: {}",
            output.status.code().unwrap_or(-1),
            if !stderr.is_empty() { stderr } else { stdout }
        ));
    }
    
    tracing::info!(boot_entry_id = entry_id, "Successfully set BootNext via pkexec/efibootmgr");
    Ok(())
}

/// Set the BootNext EFI variable to boot into a specific entry on next reboot
/// This function uses pkexec to run efibootmgr with elevated privileges
pub async fn set_boot_next(entry_id: u16) -> Result<(), String> {
    tracing::info!(boot_entry_id = entry_id, "Setting BootNext EFI variable");
    
    // Use pkexec/efibootmgr (pkexec handles polkit authentication)
    match set_boot_next_via_pkexec(entry_id).await {
        Ok(()) => return Ok(()),
        Err(e) => {
            tracing::warn!(
                boot_entry_id = entry_id,
                error = %e,
                "pkexec/efibootmgr failed, attempting direct EFI access as fallback"
            );
            // Fall back to direct EFI access (might work in some configurations)
        }
    }
    
    // Fallback: try direct EFI access (for systems where user has direct access)
    let mut adapter = Adapter::default();
    adapter
        .set_boot_next(entry_id)
        .map_err(|e| {
            let error_msg = format!("{}", e);
            // Check if it's a permission error
            if error_msg.contains("permission denied") || error_msg.contains("Permission denied") {
                tracing::error!(
                    boot_entry_id = entry_id,
                    error = %e,
                    "Both pkexec and direct EFI access failed"
                );
                format!(
                    "Permission denied: EFI variable access requires elevated privileges.\n\n\
                    Solutions:\n\
                    1. Ensure polkit policy is installed: sudo cp resources/com.github.cosmic_ext.restartTo.policy /usr/share/polkit-1/actions/\n\
                    2. Ensure efibootmgr is installed: sudo apt install efibootmgr\n\
                    3. Ensure polkitd service is running: systemctl status polkit\n\
                    4. Authenticate when pkexec prompts for your password\n\n\
                    Original error: {}",
                    error_msg
                )
            } else {
                tracing::error!(
                    boot_entry_id = entry_id,
                    error = %e,
                    "Failed to set BootNext EFI variable"
                );
                format!("Failed to set BootNext: {}", error_msg)
            }
        })?;
    tracing::info!(boot_entry_id = entry_id, "Successfully set BootNext EFI variable via direct access");
    Ok(())
}

/// Reboot the system using systemd-logind via D-Bus
pub async fn reboot_system() -> Result<(), String> {
    use zbus::Connection;

    tracing::info!("Initiating system reboot via systemd-logind");
    
    let connection = Connection::system()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to connect to system D-Bus");
            format!("Failed to connect to system D-Bus: {}", e)
        })?;

    tracing::debug!("Connected to system D-Bus");

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to create login1 proxy");
        format!("Failed to create login1 proxy: {}", e)
    })?;

    tracing::debug!("Created login1 Manager proxy");

    proxy
        .call_method("Reboot", &(true))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to call Reboot method on systemd-logind");
            format!("Failed to call Reboot method: {}", e)
        })?;

    tracing::info!("Reboot command sent successfully to systemd-logind");
    Ok(())
}
