use super::{ComputerOpResult, SecurityLevel};

pub fn set_wallpaper(path: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] wallpaper set to: {}", path),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn set_volume(level: u8) -> ComputerOpResult {
    let level = level.min(100);
    ComputerOpResult {
        success: true,
        data: format!("[simulated] volume set to: {}%", level),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn network_status() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: r#"{"status": "connected", "ssid": "Morn-Network", "signal_strength": 85}"#.into(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn power_status() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: r#"{"battery": 72, "charging": true, "time_remaining": "2h 15m"}"#.into(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
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
