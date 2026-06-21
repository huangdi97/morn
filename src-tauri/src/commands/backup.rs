use crate::AppState;
use crate::MornError;
use tauri::State;

use morn::core::storage::AgentRecord;

#[tauri::command]
pub(crate) fn export_mornpack(state: State<AppState>) -> Result<String, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let agents = s.list_agents()?;
    let pack = serde_json::json!({ "agents": agents });
    serde_json::to_string_pretty(&pack).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn import_mornpack(data: String, state: State<AppState>) -> Result<usize, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let pack: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| format!("Invalid mornpack: {}", e))?;

    let agents: Vec<AgentRecord> = serde_json::from_value(pack["agents"].clone())
        .map_err(|e| format!("Invalid agent data: {}", e))?;

    let count = agents.len();
    for agent in &agents {
        s.insert_agent(agent)?;
    }
    Ok(count)
}

#[tauri::command]
pub(crate) fn create_backup(state: State<AppState>, path: String) -> Result<String, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    let target = std::path::PathBuf::from(&path);
    s.backup_to(target.clone())
        .map_err(|e| MornError::Internal(e))?;
    Ok(format!("Backup saved to {}", target.display()))
}

#[tauri::command]
pub(crate) fn restore_backup(state: State<AppState>, path: String) -> Result<String, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    let source = std::path::PathBuf::from(&path);
    s.restore_from(source.clone())
        .map_err(|e| MornError::Internal(e))?;
    Ok(format!("Restored from {}", source.display()))
}
