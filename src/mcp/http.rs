//! MCP HTTP transport — reqwest HTTP POST JSON-RPC.
use crate::core::mcp::{MCPError, MCPResponse};

/// Call a tool via HTTP POST JSON-RPC.
pub fn call_http(
    url: &str,
    api_key: &Option<String>,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<MCPResponse, MCPError> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{}/call", url.trim_end_matches('/'));

    let mut req = client.post(&request_url).json(&serde_json::json!({
        "tool": tool_name,
        "params": args,
    }));

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let resp = req.send().map_err(|e| MCPError(format!("HTTP request failed: {e}")))?;
    let data: MCPResponse = resp.json().map_err(|e| MCPError(format!("JSON decode failed: {e}")))?;
    Ok(data)
}