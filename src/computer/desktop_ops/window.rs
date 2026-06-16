//! Window management — switch, find, and focus windows via OS APIs or simulation.

use super::run_ps;
use super::{ComputerOpResult, SecurityLevel};

/// Switch focus to window with matching title.
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
                data: e.to_string(),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        tracing::info!("[desktop_ops] window_switch simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] switched to window: {}", title),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Find window by title partial match, return window handle info.
pub fn find_window(title: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let cmd = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
$found = $false
$shell = New-Object -ComObject "Shell.Application"
$shell.Windows() | ForEach-Object {{ 
    $t = $_.Document.Title
    if ($t -like '*{}*') {{ 
        Write-Output "found: $t"
        $found = $true
    }}
}}
if (-not $found) {{ Write-Output "not found" }}
"#,
            title
        );
        match run_ps(&cmd) {
            Ok(result) => ComputerOpResult {
                success: true,
                data: result,
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
        tracing::info!("[desktop_ops] find_window simulated on non-Windows");
        ComputerOpResult {
            success: true,
            data: format!("[simulated] find window: {}", title),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
    }
}

/// Focus a specific window (activate and bring to front).
pub fn focus_window(title: &str) -> ComputerOpResult {
    window_switch(title)
}
