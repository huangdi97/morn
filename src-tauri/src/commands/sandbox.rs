use crate::AppState;
use tauri::State;

use morn::sandbox::wasm::Sandbox;

#[tauri::command]
pub(crate) fn run_in_sandbox(code: String) -> Result<String, String> {
    let sandbox = Sandbox::new().map_err(|e| format!("Sandbox init failed: {}", e))?;
    sandbox.execute(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn load_plugin_sandboxed(
    path: String,
    state: State<AppState>,
) -> Result<String, String> {
    let plugin_manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let mgr = plugin_manager
        .as_ref()
        .ok_or("PluginManager not initialized")?;
    mgr.load_plugin_sandboxed(&path)?;
    Ok(format!("Plugin loaded from {}", path))
}

#[tauri::command]
pub(crate) fn sandbox_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": true,
        "max_memory_mb": 64,
        "max_fuel": 100_000,
        "max_execution_ms": 5000,
    }))
}
