use crate::commands::errors::CommandError;

#[tauri::command]
pub(crate) fn list_local_models() -> Result<Vec<String>, CommandError> {
    Ok(vec!["llama-3.2-3b".to_string(), "qwen-2.5-7b".to_string()])
}

#[tauri::command]
pub(crate) fn download_model(name: String) -> Result<String, CommandError> {
    if name.is_empty() {
        return Err(CommandError::NotFound("model not found".to_string()));
    }
    Ok(format!("downloading {name}"))
}

#[tauri::command]
pub(crate) fn delete_local_model(name: String) -> Result<String, CommandError> {
    if name.is_empty() {
        return Err(CommandError::NotFound("model not found".to_string()));
    }
    Ok(format!("deleted {name}"))
}
