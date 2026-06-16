//! Screenshot perception — screen capture and VLM-based analysis.
use crate::core::error::MornError;
use crate::computer::{ComputerOpResult, SecurityLevel};

const DEFAULT_VLM_ENDPOINT: &str = "https://api.openai.com";

pub fn pixel_screenshot() -> ComputerOpResult {
    if cfg!(target_os = "macos") {
        let path = std::env::temp_dir().join("morn_screenshot.png");
        let result = std::process::Command::new("screencapture")
            .args(["-x", "-C", "-t", "png"])
            .arg(&path)
            .output();
        match result {
            Ok(output) if output.status.success() => {
                let data = std::fs::read(&path);
                let _ = std::fs::remove_file(&path);
                match data {
                    Ok(data) => {
                        let b64 = base64_encode(&data);
                        analyze_screenshot_result(&b64)
                    }
                    Err(e) => ComputerOpResult {
                        success: false,
                        data: format!("Screenshot captured but failed to read file: {}", e),
                        security_level: SecurityLevel::L2Local.as_str().to_string(),
                        approval_required: true,
                    },
                }
            }
            _ => ComputerOpResult {
                success: false,
                data: "Failed to capture screenshot with macOS screencapture".into(),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: true,
            },
        }
    } else if cfg!(target_os = "linux") {
        let result = std::process::Command::new("import")
            .args(["-window", "root", "png:-"])
            .output();
        match result {
            Ok(output) if output.status.success() => {
                let b64 = base64_encode(&output.stdout);
                analyze_screenshot_result(&b64)
            }
            _ => {
                let screenshot_path = std::env::temp_dir().join("morn_screenshot.png");
                let result = std::process::Command::new("scrot")
                    .args(["-o", &screenshot_path.to_string_lossy()])
                    .output();
                match result {
                    Ok(output) if output.status.success() => {
                        if let Ok(data) = std::fs::read(&screenshot_path) {
                            let b64 = base64_encode(&data);
                            return analyze_screenshot_result(&b64);
                        }
                        ComputerOpResult {
                            success: false,
                            data: format!(
                                "Screenshot captured but failed to read {}",
                                screenshot_path.to_string_lossy()
                            ),
                            security_level: SecurityLevel::L2Local.as_str().to_string(),
                            approval_required: true,
                        }
                    }
                    _ => ComputerOpResult {
                        success: false,
                        data: "Failed to capture screenshot with import or scrot".into(),
                        security_level: SecurityLevel::L2Local.as_str().to_string(),
                        approval_required: true,
                    },
                }
            }
        }
    } else {
        ComputerOpResult {
            success: false,
            data: "Real pixel screenshot is only implemented on Linux and macOS".into(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        }
    }
}

fn analyze_screenshot_result(screenshot_b64: &str) -> ComputerOpResult {
    let vlm_endpoint =
        std::env::var("MORN_VLM_ENDPOINT").unwrap_or_else(|_| DEFAULT_VLM_ENDPOINT.to_string());
    let vlm_api_key = match std::env::var("MORN_VLM_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            return ComputerOpResult {
                success: false,
                data: "MORN_VLM_API_KEY not set; real VLM screen analysis requires an API key"
                    .into(),
                security_level: SecurityLevel::L2Local.as_str().to_string(),
                approval_required: true,
            }
        }
    };

    match analyze_screen(screenshot_b64, &vlm_endpoint, &vlm_api_key) {
        Ok(data) => ComputerOpResult {
            success: true,
            data,
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        },
        Err(e) => ComputerOpResult {
            success: false,
            data: e.to_string(),
            security_level: SecurityLevel::L2Local.as_str().to_string(),
            approval_required: true,
        },
    }
}

pub fn analyze_screen(
    screenshot_b64: &str,
    vlm_endpoint: &str,
    vlm_api_key: &str,
) -> Result<String, MornError> {
    if screenshot_b64.trim().is_empty() {
        return Err(MornError::Internal("Screenshot data is empty".to_string()))
    }
    if vlm_endpoint.trim().is_empty() {
        return Err(MornError::Internal("VLM endpoint is empty".to_string()))
    }
    if vlm_api_key.trim().is_empty() {
        return Err(MornError::Internal(
            "VLM API key is empty; real VLM screen analysis requires an API key".to_string(),
        ));
    }

    let endpoint = vlm_endpoint.trim().trim_end_matches('/');
    let url = if endpoint.ends_with("/v1") {
        format!("{}/chat/completions", endpoint)
    } else {
        format!("{}/v1/chat/completions", endpoint)
    };
    let image_url = format!("data:image/png;base64,{}", screenshot_b64);
    let payload = serde_json::json!({
        "model": "gpt-4o",
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image_url",
                    "image_url": {
                        "url": image_url
                    }
                },
                {
                    "type": "text",
                    "text": "Describe what you see on this screen: windows, buttons, text labels, controls. Return JSON with keys windows, buttons, text, coordinates; include approximate bounding boxes when visible."
                }
            ]
        }]
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| MornError::Internal(format!("Failed to create VLM HTTP client: {}", e)))?;
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", vlm_api_key.trim()))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .map_err(|e| MornError::Internal(format!("VLM request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|e| MornError::Internal(format!("Failed to read VLM response body: {}", e)))?;
    if !status.is_success() {
        return Err(MornError::Internal(format!("VLM API error {}: {}", status, body)));
    }

    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| MornError::Internal(format!("Failed to parse VLM response JSON: {}", e)))?;
    let content = extract_vlm_content(&parsed)
        .ok_or_else(|| MornError::Internal("VLM response contained no message content".to_string()))?;

    Ok(format_structured_vlm_content(&content))
}

fn extract_vlm_content(response: &serde_json::Value) -> Option<String> {
    let content = response
        .get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?;

    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    content
        .as_array()?
        .iter()
        .filter_map(|part| part.get("text").and_then(|text| text.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}

fn format_structured_vlm_content(content: &str) -> String {
    if serde_json::from_str::<serde_json::Value>(content).is_ok() {
        return content.to_string();
    }

    serde_json::json!({
        "windows": [],
        "buttons": [],
        "text": [{
            "content": content,
            "coordinates": null
        }],
        "coordinates": [],
        "raw": content
    })
    .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_screenshot_returns_real_or_clear_error_result() {
        let result = pixel_screenshot();
        assert_eq!(result.security_level, SecurityLevel::L2Local.as_str());
        assert!(!result.data.is_empty());
        assert!(!result.data.contains("simulated VLM"));
    }

    #[test]
    fn base64_encodes_three_byte_chunk() {
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn base64_encodes_one_byte_padding() {
        assert_eq!(base64_encode(b"M"), "TQ==");
    }

    #[test]
    fn base64_encodes_two_byte_padding() {
        assert_eq!(base64_encode(b"Ma"), "TWE=");
    }

    #[test]
    fn format_structured_vlm_content_wraps_plain_text() {
        let structured = format_structured_vlm_content("One window with a Save button.");
        let parsed: serde_json::Value = serde_json::from_str(&structured).unwrap();
        assert!(parsed.get("windows").is_some());
        assert!(parsed.get("buttons").is_some());
        assert!(parsed.get("text").is_some());
        assert!(parsed.get("coordinates").is_some());
    }

    #[test]
    fn format_structured_vlm_content_keeps_json() {
        let content = r#"{"windows":[],"buttons":[],"text":[],"coordinates":[]}"#;
        assert_eq!(format_structured_vlm_content(content), content);
    }
}
