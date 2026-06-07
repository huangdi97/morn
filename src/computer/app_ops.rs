//! app_ops — Provides computer operations for launching and controlling applications.
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

pub fn list() -> ComputerOpResult {
    if cfg!(target_os = "windows") {
        let result = std::process::Command::new("tasklist").output();
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let processes: Vec<&str> = stdout
                    .lines()
                    .skip(3)
                    .filter_map(|l| l.split_whitespace().next())
                    .collect();
                ComputerOpResult {
                    success: true,
                    data: serde_json::to_string(&processes).unwrap_or_default(),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to list processes: {}", e),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "linux") {
        let result = std::process::Command::new("ps")
            .args(["-eo", "comm="])
            .output();
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let processes: Vec<&str> = stdout
                    .lines()
                    .filter_map(|l| {
                        let name = l.trim();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name)
                        }
                    })
                    .collect();
                ComputerOpResult {
                    success: true,
                    data: serde_json::to_string(&processes).unwrap_or_default(),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
            Err(e) => ComputerOpResult {
                success: false,
                data: format!("Failed to list processes: {}", e),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        let apps = vec!["Finder", "Terminal", "Chrome", "VSCode", "Slack", "Morn"];
        ComputerOpResult {
            success: true,
            data: serde_json::to_string(&apps).unwrap_or_default(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn install(app_path: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] installed: {}", app_path),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_invalid_name_returns_local_result() {
        let result = launch("\0");
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(result.data.contains("launch"));
    }

    #[test]
    fn close_invalid_name_returns_system_result() {
        let result = close("\0");
        assert_eq!(result.security_level, SecurityLevel::L3System.as_str());
        assert!(result.data.contains("close"));
    }

    #[test]
    fn list_returns_sandbox_result() {
        let result = list();
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(!result.data.is_empty() || !result.success);
    }

    #[test]
    fn install_requires_approval() {
        let result = install("/tmp/app.pkg");
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(result.approval_required);
        assert!(result.data.contains("/tmp/app.pkg"));
    }
}
