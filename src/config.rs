// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use std::collections::HashSet;

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    /// Set of boot entry IDs that should be hidden from the main list
    hidden_entries: HashSet<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hidden_entries: HashSet::new(),
        }
    }
}

impl Config {
    /// Check if a boot entry should be visible
    pub fn is_entry_visible(&self, entry_id: u16) -> bool {
        !self.hidden_entries.contains(&entry_id)
    }

    /// Toggle the visibility of a boot entry
    pub fn toggle_entry_visibility(&mut self, entry_id: u16) {
        if self.hidden_entries.contains(&entry_id) {
            self.hidden_entries.remove(&entry_id);
        } else {
            self.hidden_entries.insert(entry_id);
        }
    }
}
