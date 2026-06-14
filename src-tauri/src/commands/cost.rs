use crate::commands::errors::CommandError;

#[tauri::command]
pub(crate) fn estimate_cost(tokens: u64, model: String) -> Result<f64, CommandError> {
    if model.is_empty() {
        return Err(CommandError::InvalidInput("model is empty".to_string()));
    }
    Ok(0.0)
}

#[tauri::command]
pub(crate) fn get_cost_summary() -> Result<String, CommandError> {
    Ok("{\"total\": 0}".to_string())
}
