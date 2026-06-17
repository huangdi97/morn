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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_http_connection_refused() {
        let result = call_http(
            "http://127.0.0.1:1",
            &None,
            "test_tool",
            &serde_json::json!({}),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("HTTP request failed"));
    }

    #[test]
    fn test_call_http_with_api_key() {
        let result = call_http(
            "http://127.0.0.1:1",
            &Some("sk-test".to_string()),
            "test_tool",
            &serde_json::json!({"x": 1}),
        );
        assert!(result.is_err());
    }
}