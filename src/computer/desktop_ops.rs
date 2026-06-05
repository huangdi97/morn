use super::{ComputerOpResult, SecurityLevel};

pub fn mouse_move(x: i32, y: i32) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] mouse moved to ({}, {})", x, y),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn mouse_click(button: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] mouse {} click", button),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn keyboard_type(text: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] typed: {}", text),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn keyboard_hotkey(keys: &[&str]) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] hotkey: {}", keys.join("+")),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn clipboard_copy(text: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] copied to clipboard: {}", text),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn clipboard_paste() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: "[simulated] clipboard contents".into(),
        security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
        approval_required: false,
    }
}

pub fn screenshot() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: "[simulated] screenshot captured (base64)".into(),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: true,
    }
}

pub fn window_switch(title: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated] switched to window: {}", title),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}
