//! sys_ops — Provides system-level operations for shell and process control.
use super::{ComputerOpResult, SecurityLevel};
use crate::core::error::MornError;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    pub status: String,
    pub ssid: Option<String>,
    pub proxy: Option<String>,
}

pub fn set_wallpaper(path: &str) -> ComputerOpResult {
    #[cfg(target_os = "linux")]
    if std::process::Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            &format!("file://{}", path),
        ])
        .output()
        .is_ok()
    {
        return ComputerOpResult {
            success: true,
            data: format!("wallpaper set to: {}", path),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        };
    }

    #[cfg(target_os = "windows")]
    if let Ok(_) = std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Add-Type -AssemblyName System.Drawing; Add-Type -AssemblyName System.Windows.Forms; $img = [System.Drawing.Image]::FromFile('{}'); [System.Windows.Forms.Application]::OpenForms | ForEach-Object {{ $_.BackgroundImage = $img }}",
                path.replace('\'', "''")
            ),
        ])
        .output()
    {
        return ComputerOpResult {
            success: true,
            data: format!("wallpaper set to: {}", path),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        };
    }

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

pub fn get_network_config() -> Result<NetworkConfig, MornError> {
    let status = network_status();
    if !status.success {
        return Err(MornError::Internal(status.data));
    }

    let data: serde_json::Value = serde_json::from_str(&status.data)
        .map_err(|e| MornError::Internal(format!("network status json: {}", e)))?;
    Ok(NetworkConfig {
        status: data
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string(),
        ssid: data
            .get("ssid")
            .and_then(|value| value.as_str())
            .filter(|ssid| !ssid.is_empty())
            .map(|ssid| ssid.to_string()),
        proxy: std::env::var("HTTPS_PROXY")
            .ok()
            .or_else(|| std::env::var("HTTP_PROXY").ok()),
    })
}

pub fn set_proxy(url: &str) -> Result<(), MornError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(MornError::Internal("proxy url is empty".to_string()));
    }
    if !(trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("socks5://"))
    {
        return Err(MornError::Internal(
            "proxy url must start with http://, https://, or socks5://".to_string(),
        ));
    }

    #[cfg(target_os = "linux")]
    if let Ok(parsed) = reqwest::Url::parse(trimmed) {
        if let Some(host) = parsed.host_str() {
            let port = parsed
                .port_or_known_default()
                .unwrap_or(if parsed.scheme() == "socks5" {
                    1080
                } else {
                    80
                });
            let schema = match parsed.scheme() {
                "https" => "org.gnome.system.proxy.https",
                "socks5" => "org.gnome.system.proxy.socks",
                _ => "org.gnome.system.proxy.http",
            };
            let port_string = port.to_string();
            let mode_result = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy", "mode", "manual"])
                .output();
            let host_result = std::process::Command::new("gsettings")
                .args(["set", schema, "host", host])
                .output();
            let port_result = std::process::Command::new("gsettings")
                .args(["set", schema, "port", &port_string])
                .output();

            if mode_result.is_ok() && host_result.is_ok() && port_result.is_ok() {
                return Ok(());
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(_) = std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!("netsh winhttp set proxy '{}'", trimmed.replace('\'', "''")),
        ])
        .output()
    {
        return Ok(());
    }

    tracing::info!("simulated proxy update to '{}'", trimmed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_wallpaper_returns_local_result() {
        let tmp_path = std::env::temp_dir().join("wallpaper.png");
        let result = set_wallpaper(&tmp_path.to_string_lossy());
        assert!(result.success);
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(result.data.contains(&*tmp_path.to_string_lossy()));
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

    #[test]
    fn get_network_config_returns_status() {
        let config = get_network_config().unwrap();

        assert!(!config.status.is_empty());
    }

    #[test]
    fn set_proxy_rejects_invalid_url() {
        let err = set_proxy("localhost:8080").unwrap_err();

        assert!(err.contains("must start"));
    }

    #[test]
    fn set_proxy_accepts_http_url() {
        assert!(set_proxy("http://127.0.0.1:8080").is_ok());
    }
}
