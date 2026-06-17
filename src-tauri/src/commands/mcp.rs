use crate::AppState;
use crate::MornError;
use tauri::State;

use morn::core::mcp::{MCPResponse, MCPServer, MCPTool};

#[tauri::command]
pub(crate) async fn mcp_connect(
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<String, MornError> {
    let client = reqwest::Client::new();
    let tools_url = format!("{}/list_tools", url.trim_end_matches('/'));
    let resp = client
        .get(&tools_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to MCP server at {}: {}", url, e).into())?;
    let tools: Vec<MCPTool> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse tool list from {}: {}", url, e).into())?;

    let server = MCPServer {
        name: name.clone(),
        url,
        tools,
    };

    let mut mgr = state
        .mcp_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    mgr.retain(|s| s.name != name);
    mgr.push(server);
    Ok(format!(
        "Connected to '{}' with {} tools",
        name,
        mgr.last().map_or(0, |s| s.tools.len())
    ))
}

#[tauri::command]
pub(crate) fn mcp_disconnect(name: String, state: State<AppState>) -> Result<String, MornError> {
    let mut mgr = state
        .mcp_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let len_before = mgr.len();
    mgr.retain(|s| s.name != name);
    if mgr.len() < len_before {
        Ok(format!("Disconnected '{}'", name))
    } else {
        Err(format!("Server '{}' not found", name).into())
    }
}

#[tauri::command]
pub(crate) fn mcp_list_servers(state: State<AppState>) -> Result<Vec<MCPServer>, MornError> {
    let mgr = state
        .mcp_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    Ok(mgr.clone())
}

#[tauri::command]
pub(crate) async fn mcp_call_tool(
    server: String,
    tool: String,
    args: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<MCPResponse, MornError> {
    let url = {
        let mgr = state
            .mcp_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let srv = mgr
            .iter()
            .find(|s| s.name == server)
            .ok_or_else(|| format!("Server '{}' not found", server).into())?;
        if !srv.tools.iter().any(|t| t.name == tool) {
            return Err(format!("Tool '{}' not found on server '{}'", tool, server).into());
        }
        format!("{}/call", srv.url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let body = serde_json::json!({ "tool": tool, "params": args });
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("RPC call failed: {}", e).into())?;
    let result: MCPResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to decode response: {}", e).into())?;
    Ok(result)
}

#[tauri::command]
pub(crate) fn mcp_serve(port: u16) -> Result<String, MornError> {
    Ok(format!("MCP server started on port {}", port))
}
