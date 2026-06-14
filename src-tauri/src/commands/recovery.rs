use crate::commands::errors::CommandError;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn get_last_error(state: State<AppState>) -> Result<Option<String>, CommandError> {
    let storage_lock = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage_lock
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let logs = s
        .query_audit_log(None, Some("error"), 1)
        .map_err(|e| CommandError::Internal(e))?;
    if let Some(log) = logs.first() {
        let error_msg = log.details_json.as_deref().unwrap_or("unknown error");
        Ok(Some(format!(
            r#"{{"error": "{}", "time": "{}", "component": "{}"}}"#,
            error_msg, log.created_at, log.target_type
        )))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub(crate) fn retry_last_operation(state: State<AppState>) -> Result<String, CommandError> {
    let storage_lock = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage_lock
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let logs = s
        .query_audit_log(None, None, 1)
        .map_err(|e| CommandError::Internal(e))?;
    if let Some(log) = logs.first() {
        tracing::info!("Retrying last operation: {}", log.action);
        Ok(format!("retrying: {}", log.action))
    } else {
        Ok("retrying: no previous operation".to_string())
    }
}
