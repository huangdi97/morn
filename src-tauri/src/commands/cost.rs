use crate::commands::errors::CommandError;
use crate::AppState;
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
pub(crate) fn get_cost_summary(state: State<AppState>) -> Result<String, CommandError> {
    let storage_lock = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage_lock
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let executions = s
        .list_today_executions()
        .map_err(|e| CommandError::Internal(e))?;
    let calls = executions.len();
    let avg_tokens_per_call: u64 = 500;
    let total_tokens = calls as u64 * avg_tokens_per_call;
    let total_cost = (total_tokens as f64 / 1000.0) * 0.005;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    Ok(format!(
        r#"{{"total": {:.2}, "calls": {}, "date": "{}"}}"#,
        total_cost, calls, today
    ))
}
