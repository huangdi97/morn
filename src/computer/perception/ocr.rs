//! OCR perception — text extraction from images via tesseract.
use crate::computer::{ComputerOpResult, SecurityLevel};

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
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if stderr.is_empty() {
                format!("tesseract failed on '{}' with exit code {}", image_path, output.status.code().unwrap_or(-1))
            } else {
                format!("tesseract failed on '{}': {}", image_path, stderr)
            };
            ComputerOpResult {
                success: false,
                data: msg,
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: false,
            }
        }
        Err(e) => ComputerOpResult {
            success: false,
            data: format!("tesseract not available: {}", e),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_missing_file_returns_error() {
        let img_path = std::env::temp_dir().join("morn_missing_ocr_input.png");
        let result = ocr(&img_path.to_string_lossy());
        assert!(!result.data.contains("simulated"));
        assert!(!result.data.is_empty());
    }
}