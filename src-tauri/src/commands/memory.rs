use crate::commands::errors::CommandError;
use crate::AppState;
use morn::core::storage::MemoryEntry;
use tauri::State;

#[tauri::command]
pub(crate) fn list_memories(
    state: State<AppState>,
    agent_id: Option<String>,
    layer: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<MemoryEntry>, CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .clone()
        .ok_or_else(|| CommandError::Internal("Storage not initialized".to_string()))?;
    storage
        .list_memories(agent_id.as_deref(), layer.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.into())
}

#[tauri::command]
pub(crate) fn search_memories(
    state: State<AppState>,
    q: String,
) -> Result<Vec<MemoryEntry>, CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .clone()
        .ok_or_else(|| CommandError::Internal("Storage not initialized".to_string()))?;
    storage
        .search_memories(&q, None)
        .map_err(|e| e.into())
}

#[tauri::command]
pub(crate) fn delete_memory(
    state: State<AppState>,
    id: String,
) -> Result<String, CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .clone()
        .ok_or_else(|| CommandError::Internal("Storage not initialized".to_string()))?;
    storage.delete_memory(&id).map_err(|e| e.into())?;
    Ok("deleted".to_string())
}