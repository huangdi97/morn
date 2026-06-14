use crate::commands::errors::CommandError;

#[tauri::command]
pub(crate) fn get_usage_stats() -> Result<String, CommandError> {
    Ok(r#"{"agents": 0, "plugins": 0, "actions": 0}"#.to_string())
}

#[tauri::command]
pub(crate) fn get_performance_metrics() -> Result<String, CommandError> {
    Ok(r#"{"avg_response_ms": 0}"#.to_string())
}
