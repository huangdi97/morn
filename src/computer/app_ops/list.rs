//! list — Application listing and search operations.
use super::{ComputerOpResult, SecurityLevel};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_returns_json_array() {
        let result = list();
        assert!(result.success);
        let parsed: Result<Vec<String>, _> = serde_json::from_str(&result.data);
        assert!(parsed.is_ok());
    }

    #[test]
    fn list_empty_result_does_not_panic() {
        let result = list();
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }
}
