use crate::AppState;
use crate::MornError;
use tauri::State;
use morn::core::storage::InstalledItem;
use chrono::Utc;

#[tauri::command]
pub(crate) fn list_installed_items(state: State<AppState>) -> Result<Vec<InstalledItem>, MornError> {
    let storage = state.storage.lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage.as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    s.list_installed_items()
}

#[tauri::command]
pub(crate) fn toggle_installed_item(state: State<AppState>, id: String) -> Result<bool, MornError> {
    let storage = state.storage.lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage.as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    s.toggle_installed_item(&id)
}

#[tauri::command]
pub(crate) fn uninstall_installed_item(state: State<AppState>, id: String) -> Result<(), MornError> {
    let storage = state.storage.lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage.as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    s.uninstall_item(&id)
}
