use crate::commands::errors::CommandError;
use crate::AppState;
use crate::MornError;
use tauri::State;

#[tauri::command]
pub(crate) fn oauth_authorize(provider: String) -> Result<String, CommandError> {
    if provider.is_empty() {
        return Err(CommandError::InvalidInput("provider is empty".to_string()));
    }
    Ok(format!("https://{}/auth", provider))
}

#[tauri::command]
pub(crate) fn oauth_list_providers() -> Result<Vec<String>, CommandError> {
    Ok(vec![
        "google".to_string(),
        "github".to_string(),
        "slack".to_string(),
    ])
}
