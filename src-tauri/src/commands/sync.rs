use crate::AppState;
use crate::MornError;
use std::sync::{Arc, Mutex};
use tauri::State;

use morn::bridge::sync::SyncState;
use morn::core::storage::DeviceRecord;

#[tauri::command]
pub(crate) fn list_sync_devices(state: State<AppState>) -> Result<Vec<DeviceRecord>, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    s.list_devices()
}

#[tauri::command]
pub(crate) fn get_sync_status(state: State<AppState>) -> Result<serde_json::Value, MornError> {
    let pending = {
        let storage = state
            .storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let s = storage
            .as_ref()
            .ok_or_else(|| "Storage not initialized".to_string())?;
        s.list_unsynced_events()?.len()
    };

    let device_id = {
        let storage = state
            .storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let s = storage
            .as_ref()
            .ok_or_else(|| "Storage not initialized".to_string())?;
        s.get_setting("sync_device_id")
            .ok()
            .flatten()
            .unwrap_or_default()
    };

    let server_url = {
        let storage = state
            .storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let s = storage
            .as_ref()
            .ok_or_else(|| "Storage not initialized".to_string())?;
        s.get_setting("sync_server_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://localhost:3000".to_string())
    };

    Ok(serde_json::json!({
        "pending_events": pending,
        "device_id": device_id,
        "server_url": server_url,
        "engine_initialized": state.sync_engine.lock().ok().map(|g| g.is_some()).unwrap_or(false),
    }))
}

#[tauri::command]
pub(crate) fn set_sync_server_url(
    url: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    s.set_setting("sync_server_url", &url)?;

    let mut guard = state
        .sync_engine
        .lock()
        .map_err(|e| MornError::Internal(format!("lock error: {}", e)))?;
    if let Some(ref mut engine) = *guard {
        engine.sync_server_url = Some(url.clone());
    }

    Ok(url)
}
