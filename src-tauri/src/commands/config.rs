use crate::commands::errors::CommandError;
use crate::MornError;

#[tauri::command]
pub(crate) fn export_config() -> Result<String, CommandError> {
    let summary = serde_json::json!({
        "exported_at": "now",
        "version": "1.0"
    });
    serde_json::to_string_pretty(&summary).map_err(|e| CommandError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn import_config(data: String) -> Result<String, CommandError> {
    tracing::info!("Config import received: {} bytes", data.len());
    Ok("imported".to_string())
}
