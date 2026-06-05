use super::{ComputerOpResult, SecurityLevel};

pub fn launch(app_name: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] launched application: {}", app_name),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn close(app_name: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] closed application: {}", app_name),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn list() -> ComputerOpResult {
    let apps = vec!["Finder", "Terminal", "Chrome", "VSCode", "Slack", "Morn"];
    ComputerOpResult {
        success: true,
        data: serde_json::to_string(&apps).unwrap_or_default(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
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
