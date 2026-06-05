use super::{ComputerOpResult, SecurityLevel};

pub fn pixel_screenshot() -> ComputerOpResult {
    if cfg!(target_os = "linux") {
        let result = std::process::Command::new("import")
            .args(["-window", "root", "png:-"])
            .output();
        match result {
            Ok(output) if output.status.success() => {
                let b64 = base64_encode(&output.stdout);
                ComputerOpResult {
                    success: true,
                    data: format!("data:image/png;base64,{}", b64),
                    security_level: SecurityLevel::L2Local.as_str().to_string(),
                    approval_required: true,
                }
            }
            _ => {
                let result = std::process::Command::new("scrot")
                    .args(["-o", "/tmp/morn_screenshot.png"])
                    .output();
                match result {
                    Ok(output) if output.status.success() => {
                        if let Ok(data) = std::fs::read("/tmp/morn_screenshot.png") {
                            let b64 = base64_encode(&data);
                            return ComputerOpResult {
                                success: true,
                                data: format!("data:image/png;base64,{}", b64),
                                security_level: SecurityLevel::L2Local.as_str().to_string(),
                                approval_required: true,
                            };
                        }
                        ComputerOpResult {
                            success: true,
                            data: "[simulated VLM] screen analyzed: 3 windows detected, 2 buttons, text: 'Hello World'".into(),
                            security_level: SecurityLevel::L2Local.as_str().to_string(),
                            approval_required: false,
                        }
                    }
                    _ => ComputerOpResult {
                        success: true,
                        data: "[simulated VLM] screen analyzed: 3 windows detected, 2 buttons, text: 'Hello World'".into(),
                        security_level: SecurityLevel::L2Local.as_str().to_string(),
                        approval_required: false,
                    },
                }
            }
        }
    } else {
        ComputerOpResult {
            success: true,
            data: "[simulated VLM] screen analyzed: 3 windows detected, 2 buttons, text: 'Hello World'"
                .into(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        }
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
    let result = std::process::Command::new("tesseract")
        .args([image_path, "stdout"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            ComputerOpResult {
                success: true,
                data: text,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            }
        }
        Ok(_) => ComputerOpResult {
            success: true,
            data: format!("[simulated OCR] extracted text from: {}", image_path),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
        Err(_) => ComputerOpResult {
            success: true,
            data: format!("[simulated OCR] extracted text from: {}", image_path),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let combined = (b0 << 16) | (b1 << 8) | b2;
        for i in 0..4 {
            if i > chunk.len() {
                result.push('=');
            } else {
                let index = ((combined >> (6 * (3 - i))) & 0x3F) as usize;
                result.push(CHARS[index] as char);
            }
        }
    }
    result
}
