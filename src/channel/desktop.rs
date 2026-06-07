//! Desktop channel placeholder.
//! Actual Tauri integration lives in src-tauri/src/lib.rs.
//! This module provides the interface expected by the channel layer.

pub struct DesktopChannel;

impl Default for DesktopChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopChannel {
    pub fn new() -> Self {
        DesktopChannel
    }

    pub fn placeholder(&self) -> &str {
        "Desktop channel is handled via Tauri commands in src-tauri/"
    }
}
