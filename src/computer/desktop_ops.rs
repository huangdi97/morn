//! desktop_ops — Provides desktop automation operations for windows and input.
use super::{ComputerOpResult, SecurityLevel};

fn run_ps(command: &str) -> Result<String, String> {
    let output = std::process::Command::new("powershell.exe")
        .args(["-Command", command])
        .output()
        .map_err(|e| format!("PowerShell error: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell failed: {}", stderr))
    }
}

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
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] mouse moved to ({}, {})", x, y),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

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
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] mouse {} click", button),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

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
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] typed: {}", text),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

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
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] hotkey: {}", keys.join("+")),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

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
                data: e,
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] copied to clipboard: {}", text),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

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
                data: e,
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: "[simulated] clipboard contents".into(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn screenshot() -> ComputerOpResult {
    if cfg!(target_os = "windows") {
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
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: "[simulated] screenshot captured (base64)".into(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        }
    }
}

pub fn window_switch(title: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let cmd = format!(
            r#"
$shell = New-Object -ComObject "Shell.Application"
$shell.Windows() | ForEach-Object {{ if ($_.Document.Title -like '*{}*') {{ $_.Visible = $true; $_.Activate() }}}}
"#,
            title
        );
        match run_ps(&cmd) {
            Ok(_) => ComputerOpResult {
                success: true,
                data: format!("switched to window: {}", title),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: e,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] switched to window: {}", title),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
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
}
