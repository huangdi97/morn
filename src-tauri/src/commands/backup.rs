use crate::AppState;
use tauri::State;

use morn::core::storage::AgentRecord;

#[tauri::command]
pub(crate) fn export_mornpack(state: State<AppState>) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let agents = s.list_agents()?;
    let pack = serde_json::json!({ "agents": agents });
    serde_json::to_string_pretty(&pack).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn import_mornpack(data: String, state: State<AppState>) -> Result<usize, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
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
