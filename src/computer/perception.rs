use super::{ComputerOpResult, SecurityLevel};

pub fn pixel_screenshot() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: "[simulated VLM] screen analyzed: 3 windows detected, 2 buttons, text: 'Hello World'"
            .into(),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn accessibility_tree() -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: r#"[simulated a11y] root → window[1] → button[2] → text[3] → input[1]"#.into(),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}

pub fn ocr(image_path: &str) -> ComputerOpResult {
    ComputerOpResult {
        success: true,
        data: format!("[simulated OCR] extracted text from: {}", image_path),
        security_level: SecurityLevel::L2Local.as_str().to_string(),
        approval_required: false,
    }
}
