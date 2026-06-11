use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn get_system_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let console = state.console.lock().map_err(|e| e.to_string())?;
    let con = console
        .as_ref()
        .ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
    let dashboard = con.get_dashboard();
    let system_info = con.get_system_info();
    Ok(serde_json::json!({
        "dashboard": dashboard,
        "system_info": system_info
    }))
}

#[tauri::command]
pub(crate) fn get_component_topology(state: State<AppState>) -> Result<serde_json::Value, String> {
    let console = state.console.lock().map_err(|e| e.to_string())?;
    let con = console
        .as_ref()
        .ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
    let topology = con.get_topology();
    Ok(serde_json::to_value(topology).map_err(|e| e.to_string())?)
}
