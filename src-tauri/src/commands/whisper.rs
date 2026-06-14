use crate::commands::errors::CommandError;
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub(crate) fn transcribe_audio(path: String) -> Result<String, CommandError> {
    if !Path::new(&path).exists() {
        return Err(CommandError::NotFound(format!(
            "audio file not found: {}",
            path
        )));
    }
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
        .arg(&path)
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
    Ok(vec!["default".to_string()])
}
