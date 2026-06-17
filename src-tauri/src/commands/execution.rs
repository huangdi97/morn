use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn get_recent_logs(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let execs = s.list_recent_executions(5).map_err(|e| e.to_string())?;
    let logs: Vec<_> = execs
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "action": e.action,
                "status": e.status,
                "agent_id": e.agent_id,
                "latency_ms": e.latency_ms,
                "error_msg": e.error_msg,
                "created_at": e.created_at,
            })
        })
        .collect();
    Ok(logs)
}
