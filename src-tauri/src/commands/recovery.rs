use crate::commands::errors::CommandError;

#[tauri::command]
pub(crate) fn get_last_error() -> Result<Option<String>, CommandError> {
    Ok(None)
}

#[tauri::command]
pub(crate) fn retry_last_operation() -> Result<String, CommandError> {
    tracing::info!("Retrying last operation");
    Ok("retried".to_string())
}
