use crate::MornError;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn get_reliability_metrics(state: State<AppState>) -> Result<serde_json::Value, MornError> {
    let storage = state.storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let metrics = s.get_reliability_metrics()?;
    let recent = s.list_recent_executions(10)?;
    Ok(serde_json::json!({
        "metrics": metrics,
        "recent": recent
    }))
}
