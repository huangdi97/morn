use crate::AppState;
use tauri::State;

use morn::core::mcp::MCPServer;

#[tauri::command]
pub(crate) fn mcp_connect(
    name: String,
    url: String,
    state: State<AppState>,
) -> Result<String, String> {
    let mut mgr = state.mcp_manager.lock().map_err(|e| e.to_string())?;
    mgr.push(MCPServer {
        name,
        url,
        tools: vec![],
    });
    Ok("Connected".to_string())
}

#[tauri::command]
pub(crate) fn mcp_disconnect(name: String, state: State<AppState>) -> Result<String, String> {
    let mut mgr = state.mcp_manager.lock().map_err(|e| e.to_string())?;
    mgr.retain(|s| s.name != name);
    Ok("Disconnected".to_string())
}

#[tauri::command]
pub(crate) fn mcp_list_servers(state: State<AppState>) -> Result<Vec<MCPServer>, String> {
    let mgr = state.mcp_manager.lock().map_err(|e| e.to_string())?;
    Ok(mgr.clone())
}

#[tauri::command]
pub(crate) fn mcp_serve(port: u16) -> Result<String, String> {
    Ok(format!("MCP server started on port {}", port))
}
