use crate::commands::errors::CommandError;
use std::path::Path;

#[tauri::command]
pub(crate) fn transcribe_audio(path: String) -> Result<String, CommandError> {
    if !Path::new(&path).exists() {
        return Err(CommandError::NotFound(format!(
            "audio file not found: {}",
            path
        )));
    }
    Ok("transcribed text placeholder".to_string())
}

#[tauri::command]
pub(crate) fn list_audio_devices() -> Result<Vec<String>, CommandError> {
    Ok(vec!["default".to_string()])
}
