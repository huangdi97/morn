//! Mouse operations — move, click, and click-at via OS APIs or simulation.

use super::run_ps;
use super::{ComputerOpResult, SecurityLevel};

/// Move mouse to absolute screen coordinates (x, y).
pub fn mouse_move(x: i32, y: i32) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let cmd = format!(
            "[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({}, {})",
            x, y
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("mouse moved to ({}, {})", x, y),
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
        tracing::info!("[desktop_ops] mouse_move simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] mouse moved to ({}, {})", x, y),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Click mouse at current position. On Windows, uses user32 mouse_event.
pub fn mouse_click(button: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let btn_flag = match button {
            "right" => "0x8",
            "middle" => "0x20",
            _ => "0x6",
        };
        let cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Cursor]::Position = [System.Windows.Forms.Cursor]::Position; Start-Sleep -Milliseconds 50; \
             $sig = '[DllImport(\"user32.dll\")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo);'; \
             $type = Add-Type -MemberDefinition $sig -Name mouse -Namespace Win32 -PassThru; \
             $type::mouse_event({}, 0, 0, 0, 0); $type::mouse_event(0x2, 0, 0, 0, 0);",
            btn_flag
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("mouse {} click", button),
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
        tracing::info!("[desktop_ops] mouse_click simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] mouse {} click", button),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Click mouse at specific coordinates (x, y).
/// On Windows with `desktop-real`, moves cursor then clicks.
#[cfg(feature = "desktop-real")]
pub fn mouse_click_at(x: i32, y: i32, button: &str) -> ComputerOpResult {
    let move_result = mouse_move(x, y);
    if !move_result.success {
        return move_result;
    }
    mouse_click(button)
}

/// Stub for mouse_click_at when desktop-real is not enabled.
#[cfg(not(feature = "desktop-real"))]
pub fn mouse_click_at(x: i32, y: i32, button: &str) -> ComputerOpResult {
    tracing::info!("[desktop_ops] mouse_click_at simulated (enable desktop-real for real ops)");
    ComputerOpResult {
        success: true,
        data: format!("[simulated] mouse {} click at ({}, {})", button, x, y),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}
