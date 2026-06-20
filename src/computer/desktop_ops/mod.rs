//! desktop_ops — Provides desktop automation operations for windows and input.
//!
//! ## Platform Limitations
//! - **Windows**: Full support via PowerShell/user32.dll.
//! - **macOS**: Limited support (requires accessibility permissions).
//! - **Linux**: X11-based support via xdotool/xrandr (limited).
//! - **Other**: All operations return simulated results with log hints.
//!
//! When the `desktop-real` feature is enabled, real OS-level operations are used
//! (screenshot via `screenshots` crate, window management via OS APIs).

use crate::core::error::MornError;
pub mod keyboard;
pub mod mouse;
pub mod window;

pub use keyboard::*;
pub use mouse::*;
pub use window::*;

use super::{ComputerOpResult, SecurityLevel};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

pub(crate) fn run_ps(command: &str) -> Result<String, MornError> {
    let output = std::process::Command::new("powershell.exe")
        .args(["-Command", command])
        .output()
        .map_err(|e| MornError::Internal(format!("PowerShell error: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(MornError::Internal(format!(
            "PowerShell failed: {}",
            stderr
        )))
    }
}

/// Capture screenshot. With `desktop-real`, uses OS-native approaches.
/// On Windows, uses PowerShell via System.Drawing.
/// On macOS, uses screencapture utility.
/// On Linux, uses import (ImageMagick) or gnome-screenshot.
/// Falls back to simulation if unavailable.
pub fn screenshot() -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        return screenshot_windows_ps();
    }
    #[cfg(feature = "desktop-real")]
    {
        screenshot_native()
    }
    #[cfg(not(feature = "desktop-real"))]
    {
        tracing::info!(
            "[desktop_ops] screenshot simulated (enable desktop-real for real captures)"
        );
        ComputerOpResult {
            success: true,
            data: "[simulated] screenshot captured (base64)".into(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        }
    }
}

fn screenshot_windows_ps() -> ComputerOpResult {
    let cmd = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.X, $screen.Y, 0, 0, $screen.Size)
$ms = New-Object System.IO.MemoryStream
$bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$base64 = [System.Convert]::ToBase64String($ms.ToArray())
Write-Output $base64
"#;
    match run_ps(cmd) {
        Ok(b64) => ComputerOpResult {
            success: true,
            data: format!("data:image/png;base64,{}", b64),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: e.to_string(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

#[cfg(feature = "desktop-real")]
fn screenshot_native() -> ComputerOpResult {
    let ss_path = std::env::temp_dir().join("morn_screenshot.png");
    let output = if cfg!(target_os = "macos") {
        std::process::Command::new("screencapture")
            .args(["-x", "-C", "-t", "png", &ss_path.to_string_lossy()])
            .output()
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("import")
            .args(["-window", "root", &ss_path.to_string_lossy()])
            .output()
            .or_else(|_| {
                std::process::Command::new("gnome-screenshot")
                    .args(["-f", &ss_path.to_string_lossy()])
                    .output()
            })
    } else {
        return ComputerOpResult {
            success: false,
            data: "unsupported platform for native screenshot".to_string(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        };
    };
    match output {
        Ok(o) if o.status.success() => {
            if let Ok(data) = std::fs::read(&ss_path) {
                let b64 = base64_engine(&data);
                let _ = std::fs::remove_file(&ss_path);
                ComputerOpResult {
                    success: true,
                    data: format!("data:image/png;base64,{}", b64),
                    security_level: SecurityLevel::L2Local.as_str().to_string(),
                    approval_required: true,
                }
            } else {
                ComputerOpResult {
                    success: false,
                    data: "failed to read screenshot file".to_string(),
                    security_level: SecurityLevel::L2Local.as_str().to_string(),
                    approval_required: false,
                }
            }
        }
        Ok(_) => ComputerOpResult {
            success: false,
            data: "screenshot command failed".to_string(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("screenshot command not found: {}", e),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

#[cfg(feature = "desktop-real")]
fn base64_engine(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk.first().copied().unwrap_or(0) as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[(triple >> 18) & 0x3F] as char);
        result.push(CHARS[(triple >> 12) & 0x3F] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(triple >> 6) & 0x3F] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[triple & 0x3F] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn get_screen_resolution() -> Result<Resolution, MornError> {
    if cfg!(target_os = "windows") {
        let output = run_ps(
            "Add-Type -AssemblyName System.Windows.Forms; $b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds; Write-Output \"$($b.Width)x$($b.Height)\"",
        )?;
        return parse_resolution(&output)
            .ok_or_else(|| MornError::Internal("failed to parse resolution".to_string()));
    }

    if cfg!(target_os = "linux") {
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                "xrandr --current 2>/dev/null | sed -n 's/.*current \\([0-9]*\\) x \\([0-9]*\\).*/\\1x\\2/p' | head -1",
            ])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(resolution) = parse_resolution(&stdout) {
                return Ok(resolution);
            }
        }
    }

    Ok(Resolution {
        width: 1920,
        height: 1080,
    })
}

fn parse_resolution(value: &str) -> Option<Resolution> {
    let trimmed = value.trim();
    let (width, height) = trimmed.split_once('x')?;
    Some(Resolution {
        width: width.trim().parse().ok()?,
        height: height.trim().parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_level(result: &ComputerOpResult, level: SecurityLevel) {
        assert_eq!(result.security_level, level.as_str());
    }

    #[test]
    fn mouse_move_returns_local_result() {
        let result = mouse_move(10, 20);
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("10"));
        assert!(result.data.contains("20"));
    }

    #[test]
    fn mouse_click_defaults_to_local_control() {
        let result = mouse_click("left");
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("left"));
    }

    #[test]
    fn mouse_click_accepts_right_button() {
        let result = mouse_click("right");
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("right"));
    }

    #[test]
    fn mouse_click_at_returns_coordinates() {
        let result = mouse_click_at(100, 200, "left");
        assert!(result.data.contains("100") || result.data.contains("simulated"));
    }

    #[test]
    fn keyboard_type_returns_typed_text() {
        let result = keyboard_type("hello");
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("hello"));
    }

    #[test]
    fn keyboard_hotkey_joins_keys() {
        let result = keyboard_hotkey(&["ctrl", "shift", "p"]);
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("ctrl+shift+p"));
    }

    #[test]
    fn clipboard_copy_is_sandbox_level() {
        let result = clipboard_copy("clip");
        assert_level(&result, SecurityLevel::L1Sandbox);
        assert!(result.data.contains("clip"));
    }

    #[test]
    fn clipboard_paste_is_sandbox_level() {
        let result = clipboard_paste();
        assert_level(&result, SecurityLevel::L1Sandbox);
    }

    #[test]
    fn screenshot_requires_approval() {
        let result = screenshot();
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.approval_required || !result.success);
    }

    #[test]
    fn window_switch_returns_requested_title() {
        let result = window_switch("Terminal");
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("Terminal"));
    }

    #[test]
    fn find_window_returns_title() {
        let result = find_window("TestApp");
        assert!(result.data.contains("TestApp") || result.data.contains("simulated"));
    }

    #[test]
    fn focus_window_delegates_to_switch() {
        let result = focus_window("Browser");
        assert_level(&result, SecurityLevel::L2Local);
        assert!(result.data.contains("Browser"));
    }

    #[test]
    fn get_screen_resolution_returns_positive_size() {
        let resolution = get_screen_resolution().unwrap();
        assert!(resolution.width > 0);
        assert!(resolution.height > 0);
    }

    #[test]
    fn parse_resolution_accepts_width_by_height() {
        let resolution = parse_resolution("1280x720").unwrap();
        assert_eq!(resolution.width, 1280);
        assert_eq!(resolution.height, 720);
    }
}