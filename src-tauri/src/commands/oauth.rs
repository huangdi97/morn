use crate::commands::errors::CommandError;
use crate::AppState;
use morn::core::oauth::{OAuthConfig, ProviderInfo};
use tauri::State;

fn lock_oauth<'a>(
    state: &'a State<'_, AppState>,
) -> Result<std::sync::MutexGuard<'a, Option<morn::core::oauth::OAuthManager>>, CommandError> {
    state
        .oauth_manager
        .lock()
        .map_err(|e| CommandError::Internal(format!("OAuth lock error: {}", e)))
}

fn get_manager<'a>(
    state: &'a State<'_, AppState>,
) -> Result<std::sync::MutexGuard<'a, Option<morn::core::oauth::OAuthManager>>, CommandError> {
    let guard = lock_oauth(state)?;
    if guard.is_none() {
        return Err(CommandError::Internal("OAuth not initialized".into()));
    }
    Ok(guard)
}

#[tauri::command]
pub(crate) fn oauth_authorize(
    state: State<'_, AppState>,
    provider: String,
) -> Result<String, CommandError> {
    let guard = get_manager(&state)?;
    let manager = guard.as_ref().unwrap();
    manager
        .get_auth_url(&provider)
        .map_err(|e| CommandError::Internal(format!("Failed to get auth URL: {}", e)))
}

#[tauri::command]
pub(crate) fn oauth_callback(
    state: State<'_, AppState>,
    provider: String,
    code: String,
) -> Result<String, CommandError> {
    let guard = get_manager(&state)?;
    let manager = guard.as_ref().unwrap();
    let token = manager
        .handle_callback(&provider, &code)
        .map_err(|e| CommandError::Network(format!("OAuth callback failed: {}", e)))?;
    Ok(token.access_token)
}

#[tauri::command]
pub(crate) fn oauth_list_providers(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderInfo>, CommandError> {
    let guard = get_manager(&state)?;
    let manager = guard.as_ref().unwrap();
    Ok(manager.list_provider_info())
}

#[tauri::command]
pub(crate) fn oauth_save_config(
    state: State<'_, AppState>,
    provider: String,
    client_id: String,
    client_secret: String,
) -> Result<(), CommandError> {
    let mut guard = lock_oauth(&state)?;
    let manager = guard
        .as_mut()
        .ok_or_else(|| CommandError::Internal("OAuth not initialized".into()))?;
    manager
        .set_provider_credentials(&provider, client_id, client_secret)
        .map_err(|e| CommandError::Internal(format!("Failed to save config: {}", e)))
}
