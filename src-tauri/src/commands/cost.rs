use crate::commands::errors::CommandError;
use crate::AppState;
use crate::MornError;
use tauri::State;

fn price_per_1k(model: &str) -> f64 {
    let m = model.to_ascii_lowercase();
    if m.contains("gpt") {
        0.01
    } else if m.contains("deepseek") {
        0.001
    } else {
        0.005
    }
}

#[tauri::command]
pub(crate) fn estimate_cost(tokens: u64, model: String) -> Result<f64, CommandError> {
    if model.is_empty() {
        return Err(CommandError::InvalidInput("model is empty".to_string()));
    }
    Ok((tokens as f64 / 1000.0) * price_per_1k(&model))
}

#[tauri::command]
pub(crate) fn get_cost_summary(state: State<AppState>) -> Result<serde_json::Value, CommandError> {
    let storage_lock = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage_lock
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let summary = s
        .get_cost_summary(7)
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    Ok(serde_json::json!({
        "total": summary.total_cost,
        "calls": summary.total_calls,
        "tokens": summary.total_tokens,
        "by_date": summary.by_date,
    }))
}

#[tauri::command]
pub(crate) fn get_cost_details(
    state: State<AppState>,
    agent_id: Option<String>,
) -> Result<serde_json::Value, CommandError> {
    let storage_lock = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage_lock
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let rows = if let Some(aid) = agent_id {
        s.get_agent_costs(&aid, 7)
            .map_err(|e| CommandError::Internal(e.to_string()))?
    } else {
        s.get_cost_summary(7)
            .map_err(|e| CommandError::Internal(e.to_string()))?
            .by_date
    };
    Ok(serde_json::json!(rows))
}
