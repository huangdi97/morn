//! Desktop channel placeholder.
//! Actual Tauri integration lives in src-tauri/src/lib.rs.
//! This module provides the interface expected by the channel layer.

use crate::core::error::MornError;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_desktop_channel() {
        let channel = DesktopChannel::new();
        assert!(channel.placeholder().contains("Tauri"));
    }

    #[test]
    fn default_matches_new_placeholder() {
        let new_channel = DesktopChannel::new();
        let default_channel = DesktopChannel;
        assert_eq!(new_channel.placeholder(), default_channel.placeholder());
    }

    #[test]
    fn placeholder_mentions_channel_owner() {
        let channel = DesktopChannel::default();
        assert!(channel.placeholder().contains("src-tauri"));
    }
}
