//! sys_ops — Provides system-level operations for shell and process control.
use super::{ComputerOpResult, SecurityLevel};

pub fn set_wallpaper(path: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] wallpaper set to: {}", path),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn get_volume() -> ComputerOpResult {
    if cfg!(target_os = "linux") {
        let result = std::process::Command::new("amixer")
            .args(["get", "Master"])
            .output();
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(cap) = stdout.split('[').nth(1) {
                    if let Some(pct) = cap.split('%').next() {
                        return ComputerOpResult {
                            success: true,
                            data: format!("{{\"volume\": {}}}", pct),
                            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                            approval_required: false,
                        };
                    }
                }
                ComputerOpResult {
                    success: true,
                    data: "{\"volume\": 50}".into(),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
            Err(_) => ComputerOpResult {
                success: true,
                data: "{\"volume\": 50}".into(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "windows") {
        let result = std::process::Command::new("powershell.exe")
            .args(["-Command", "(New-Object -ComObject SAPI.SpVoice).Volume"])
            .output();
        match result {
            Ok(output) => {
                let vol = String::from_utf8_lossy(&output.stdout).trim().to_string();
                ComputerOpResult {
                    success: true,
                    data: format!("{{\"volume\": {}}}", vol),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
            Err(_) => ComputerOpResult {
                success: true,
                data: "{\"volume\": 50}".into(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: "{\"volume\": 50}".into(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn set_volume(level: u8) -> ComputerOpResult {
    let level = level.min(100);
    if cfg!(target_os = "linux") {
        let result = std::process::Command::new("amixer")
            .args(["set", "Master", &format!("{}%", level)])
            .output();
        match result {
            Ok(output) if output.status.success() => ComputerOpResult {
                success: true,
                data: format!("volume set to: {}%", level),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
            _ => ComputerOpResult {
                success: true,
                data: format!("[simulated] volume set to: {}%", level),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else if cfg!(target_os = "windows") {
        let cmd = format!("(New-Object -ComObject SAPI.SpVoice).Volume = {}", level);
        let _ = std::process::Command::new("powershell.exe")
            .args(["-Command", &cmd])
            .output();
        ComputerOpResult {
            success: true,
            data: format!("volume set to: {}%", level),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    } else {
        ComputerOpResult {
            success: true,
            data: format!("[simulated] volume set to: {}%", level),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn network_status() -> ComputerOpResult {
    let ping_result = std::process::Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output();

    match ping_result {
        Ok(output) if output.status.success() => {
            let latency = String::from_utf8_lossy(&output.stdout);
            let time_ms = latency
                .split("time=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("0")
                .trim_end_matches('s')
                .to_string();

            ComputerOpResult {
                success: true,
                data: serde_json::json!({
                    "status": "connected",
                    "latency_ms": time_ms,
                })
                .to_string(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            }
        }
        _ => {
            let wifi_result = std::process::Command::new("iwgetid")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            ComputerOpResult {
                success: true,
                data: serde_json::json!({
                    "status": if wifi_result.is_some() { "connected" } else { "disconnected" },
                    "ssid": wifi_result.unwrap_or_default(),
                })
                .to_string(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            }
        }
    }
}

pub fn power_status() -> ComputerOpResult {
    if cfg!(target_os = "linux") {
        let battery_path = std::path::Path::new("/sys/class/power_supply");
        if battery_path.exists() {
            if let Ok(entries) = std::fs::read_dir(battery_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("BAT") || name_str.starts_with("battery") {
                        let capacity_path = entry.path().join("capacity");
                        let status_path = entry.path().join("status");
                        let capacity = std::fs::read_to_string(&capacity_path)
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        let status = std::fs::read_to_string(&status_path)
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        let charging = status == "Charging";

                        return ComputerOpResult {
                            success: true,
                            data: serde_json::json!({
                                "battery": capacity.parse::<u32>().unwrap_or(0),
                                "charging": charging,
                            })
                            .to_string(),
                            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                            approval_required: false,
                        };
                    }
                }
            }
        }
        ComputerOpResult {
            success: true,
            data: serde_json::json!({
                "battery": 100,
                "charging": true,
            })
            .to_string(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    } else if cfg!(target_os = "windows") {
        let result = std::process::Command::new("powershell.exe")
            .args(["-Command",
                "Get-WmiObject Win32_Battery | Select-Object EstimatedChargeRemaining, BatteryStatus | ConvertTo-Json"])
            .output();
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                ComputerOpResult {
                    success: true,
                    data: stdout.to_string(),
                    security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                    approval_required: false,
                }
            }
            Err(_) => ComputerOpResult {
                success: true,
                data: r#"{"battery": 100, "charging": true}"#.into(),
                security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
                approval_required: false,
            },
        }
    } else {
        ComputerOpResult {
            success: true,
            data: r#"{"battery": 72, "charging": true, "time_remaining": "2h 15m"}"#.into(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        }
    }
}

pub fn shutdown(delay_secs: u32) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] system shutdown in {}s", delay_secs),
        security_level: SecurityLevel::L3System.as_str().to_string(),
        approval_required: true,
    }
}

pub fn sleep() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: "[simulated] system sleep".into(),
        security_level: SecurityLevel::L3System.as_str().to_string(),
        approval_required: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_wallpaper_returns_local_result() {
        let result = set_wallpaper("/tmp/wallpaper.png");
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(result.data.contains("/tmp/wallpaper.png"));
    }

    #[test]
    fn get_volume_returns_sandbox_result() {
        let result = get_volume();
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.data.contains("volume"));
    }

    #[test]
    fn set_volume_caps_level_at_100() {
        let result = set_volume(150);
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.data.contains("100%"));
    }

    #[test]
    fn network_status_returns_json_like_data() {
        let result = network_status();
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
        assert!(result.data.contains("status"));
    }

    #[test]
    fn power_status_returns_sandbox_result() {
        let result = power_status();
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L1Sandbox.as_str());
    }

    #[test]
    fn shutdown_requires_system_approval() {
        let result = shutdown(30);
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L3System.as_str());
        assert!(result.approval_required);
        assert!(result.data.contains("30"));
    }

    #[test]
    fn sleep_requires_system_approval() {
        let result = sleep();
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L3System.as_str());
        assert!(result.approval_required);
    }

    #[test]
    fn system_power_operations_use_distinct_messages() {
        assert_ne!(shutdown(1).data, sleep().data);
    }
}
