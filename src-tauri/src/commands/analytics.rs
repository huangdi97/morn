use crate::MornError;
use crate::commands::errors::CommandError;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn get_usage_stats(state: State<AppState>) -> Result<String, CommandError> {
    let storage = state.storage.lock().map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not initialized".to_string()))?;
    let executions = s.list_today_executions().map_err(|e| CommandError::Internal(e))?;
    let agents = executions.iter().map(|e| e.agent_id.clone()).collect::<std::collections::HashSet<_>>().len();
    let total_calls = executions.len();
    let success_calls = executions.iter().filter(|e| e.status == "success").count();
    Ok(serde_json::json!({
        "agents": agents,
        "calls": total_calls,
        "success_calls": success_calls,
    }).to_string())
}

#[tauri::command]
pub(crate) fn get_performance_metrics() -> Result<String, CommandError> {
    Ok(r#"{"avg_response_ms": 0}"#.to_string())
}
