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
    fn test_install_simulated() {
        let result = install("/usr/bin/test");
        assert!(result.success);
        assert!(result.data.contains("test"));
        assert_eq!(result.security_level, "local");
        assert!(result.approval_required);
    }

    #[test]
    fn test_launch_simulated_fallback() {
        if !cfg!(target_os = "windows") && !cfg!(target_os = "linux") {
            let result = launch("test_app");
            assert!(result.success);
            assert!(result.data.contains("test_app"));
        }
    }

    #[test]
    fn test_list_returns_apps_on_fallback() {
        if !cfg!(target_os = "windows") && !cfg!(target_os = "linux") {
            let result = list();
            assert!(result.success);
            let apps: Vec<String> = serde_json::from_str(&result.data).unwrap();
            assert!(apps.contains(&"Morn".to_string()));
        }
    }
}
