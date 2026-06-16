//! Keyboard operations — typing, hotkeys, and clipboard management via OS APIs or simulation.

use crate::core::error::MornError;
use super::run_ps;
use super::{ComputerOpResult, SecurityLevel};

/// Type text via keyboard. On Windows, uses SendKeys.
pub fn keyboard_type(text: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let escaped = text.replace('\'', "''");
        let cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')",
            escaped
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("typed: {}", text),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: e.to_string(),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        tracing::info!("[desktop_ops] keyboard_type simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] typed: {}", text),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Send keyboard hotkey combination (e.g. ctrl+shift+p).
pub fn keyboard_hotkey(keys: &[&str]) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let combo = keys.join("+");
        let cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('({})')",
            combo
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("hotkey: {}", keys.join("+")),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: e.to_string(),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        tracing::info!("[desktop_ops] keyboard_hotkey simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] hotkey: {}", keys.join("+")),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Copy text to clipboard.
pub fn clipboard_copy(text: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let escaped = text.replace('\'', "''");
        let cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Clipboard]::SetText('{}')",
            escaped
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("copied to clipboard: {}", text),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: e.to_string(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        tracing::info!("[desktop_ops] clipboard_copy simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] copied to clipboard: {}", text),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Paste text from clipboard.
pub fn clipboard_paste() -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let cmd = "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Clipboard]::GetText()";
        match run_ps(cmd) {
            Ok(text) => ComputerOpResult {
                success: true,
                data: text,
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: e.to_string(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        tracing::info!("[desktop_ops] clipboard_paste simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: "[simulated] clipboard contents".into(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_type_simulated() {
        let result = keyboard_type("hello world");
        assert!(result.success);
        assert!(result.data.contains("[simulated]"));
        assert!(result.data.contains("hello world"));
    }

    #[test]
    fn test_keyboard_type_empty() {
        let result = keyboard_type("");
        assert!(result.success);
        assert!(result.data.contains("[simulated]"));
    }

    #[test]
    fn test_keyboard_hotkey_simulated() {
        let result = keyboard_hotkey(&["ctrl", "shift", "p"]);
        assert!(result.success);
        assert!(result.data.contains("[simulated]"));
        assert!(result.data.contains("ctrl+shift+p"));
    }

    #[test]
    fn test_keyboard_hotkey_single_key() {
        let result = keyboard_hotkey(&["enter"]);
        assert!(result.success);
        assert!(result.data.contains("enter"));
    }

    #[test]
    fn test_clipboard_copy_simulated() {
        let result = clipboard_copy("test content");
        assert!(result.success);
        assert!(result.data.contains("[simulated]"));
        assert!(result.data.contains("test content"));
    }

    #[test]
    fn test_clipboard_copy_empty() {
        let result = clipboard_copy("");
        assert!(result.success);
    }

    #[test]
    fn test_clipboard_paste_simulated() {
        let result = clipboard_paste();
        assert!(result.success);
        assert!(result.data.contains("[simulated]"));
    }

    #[test]
    fn test_security_levels() {
        let typing = keyboard_type("x");
        assert_eq!(typing.security_level, "local");

        let hotkey = keyboard_hotkey(&["a"]);
        assert_eq!(hotkey.security_level, "local");

        let copy = clipboard_copy("x");
        assert_eq!(copy.security_level, "sandbox");

        let paste = clipboard_paste();
        assert_eq!(paste.security_level, "sandbox");
    }

    #[test]
    fn test_no_approval_required() {
        assert!(!keyboard_type("x").approval_required);
        assert!(!keyboard_hotkey(&["a"]).approval_required);
        assert!(!clipboard_copy("x").approval_required);
        assert!(!clipboard_paste().approval_required);
    }
}
