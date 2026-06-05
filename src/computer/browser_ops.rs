use super::{ComputerOpResult, SecurityLevel};

pub fn navigate(url: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] navigated to: {}", url),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn form_fill(selector: &str, value: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] filled '{}' with '{}'", selector, value),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn content_extract(url: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] extracted content from: {}", url),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn multi_tab(tabs: &[&str]) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] opened {} tabs", tabs.len()),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}
