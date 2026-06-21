use crate::commands::errors::CommandError;
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub(crate) fn transcribe_audio(
    path: Option<String>,
    data: Option<String>,
) -> Result<String, CommandError> {
    let audio_path = if let Some(data) = data {
        let bytes = base64::decode(&data)
            .map_err(|e| CommandError::InvalidInput(format!("base64 decode failed: {}", e)))?;
        let tmp_path = std::env::temp_dir().join(format!("morn_voice_{}.webm", uuid::Uuid::new_v4()));
        std::fs::write(&tmp_path, &bytes)
            .map_err(|e| CommandError::Internal(format!("failed to write temp file: {}", e)))?;
        tmp_path
    } else if let Some(path) = path {
        if !Path::new(&path).exists() {
            return Err(CommandError::NotFound(format!(
                "audio file not found: {}",
                path
            )));
        }
        path.into()
    } else {
        return Err(CommandError::InvalidInput(
            "No audio data provided".to_string(),
        ));
    };

    let exists = Command::new("which")
        .arg("whisper")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !exists {
        return Err(CommandError::Internal(
            "whisper not installed. Install via: pip install openai-whisper".to_string(),
        ));
    }
    let output = Command::new("whisper")
        .arg(&audio_path)
        .arg("--output_format")
        .arg("txt")
        .arg("--language")
        .arg("zh")
        .output()
        .map_err(|e| CommandError::Internal(format!("failed to run whisper: {}", e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Internal(format!(
            "whisper failed: {}",
            stderr
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

#[tauri::command]
pub(crate) fn list_audio_devices() -> Result<Vec<String>, CommandError> {
    let mut devices = vec!["default".to_string()];

    if let Ok(output) = Command::new("arecord").args(["-l"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(rest) = line.strip_prefix("card ") {
                    let parts: Vec<&str> = rest.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let card_str = parts[0].trim();
                        if let Some(dev_part) = parts[1].split(',').next() {
                            if let Some(device_str) = dev_part
                                .trim()
                                .strip_prefix("device ")
                                .map(|s| s.trim())
                            {
                                devices.push(format!("hw:{},{}", card_str, device_str));
                                devices.push(format!("plughw:{},{}", card_str, device_str));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(devices)
}
