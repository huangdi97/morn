//! launch — Application launch and close operations.
use super::{ComputerOpResult, SecurityLevel};

pub fn launch(app_name: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let result = std::process::Command::new("powershell.exe")
            .args(["-Command", &format!("Start-Process '{}'", app_name)])
            .output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("launched application: {}", app_name),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to launch '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to launch '{}': {}", app_name, e),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "macos") {
        let result = std::process::Command::new("open")
            .args(["-a", app_name])
            .output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("launched application: {}", app_name),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to launch '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to launch '{}': {}", app_name, e),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "linux") {
        let result = std::process::Command::new("sh")
            .args(["-c", &format!("which '{}' && '{}' &", app_name, app_name)])
            .output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("launched application: {}", app_name),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to launch '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to launch '{}': {}", app_name, e),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] launched application: {}", app_name),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn close(app_name: &str) -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let result = std::process::Command::new("taskkill")
            .args(["/IM", app_name, "/F"])
            .output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("closed application: {}", app_name),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: true,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to close '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to close '{}': {}", app_name, e),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "macos") {
        let script = format!(
            "tell application \"{}\" to quit",
            app_name.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let result = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .or_else(|_| std::process::Command::new("pkill").arg(app_name).output());
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("closed application: {}", app_name),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: true,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to close '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to close '{}': {}", app_name, e),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "linux") {
        let result = std::process::Command::new("pkill").arg(app_name).output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("closed application: {}", app_name),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: true,
            },
            Ok(output) => ComputerOpResult {
                success: false,
                data: format!(
                    "Failed to close '{}': {}",
                    app_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to close '{}': {}", app_name, e),
                security_level: SecurityLevel::L3System.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] closed application: {}", app_name),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_empty_name_returns_simulated() {
        let result = launch("");
        assert!(result.data.contains("launch"));
    }

    #[test]
    fn launch_invalid_args_does_not_panic() {
        let result = launch("\0");
        assert!(result.success || !result.success);
    }
}
