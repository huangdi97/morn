//! mcp — Provides Model Context Protocol server and tool integration support.
use crate::core::error::MornError;
use std::sync::{Arc, Mutex};

use crate::core::registry::Registry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPRequest {
    pub tool: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub server_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPServer {
    pub name: String,
    pub url: String,
    pub tools: Vec<MCPTool>,
}

#[derive(Debug, Clone)]
pub struct MCPError(pub String);

impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MCPError: {}", self.0)
    }
}

#[derive(Clone)]
pub struct MCPClient {
    http_client: reqwest::blocking::Client,
    registry: Arc<Mutex<Registry>>,
    servers: Vec<MCPServer>,
}

impl MCPClient {
    /// Creates an MCP client backed by a shared registry.
    pub fn new(registry: Arc<Mutex<Registry>>) -> Self {
        MCPClient {
            http_client: reqwest::blocking::Client::new(),
            registry,
            servers: Vec::new(),
        }
    }

    /// Calls a registered MCP tool with request parameters and returns the server response.
    pub fn call_tool(&self, request: MCPRequest) -> Result<MCPResponse, MCPError> {
        for server in &self.servers {
            if server.tools.iter().any(|t| t.name == request.tool) {
                let url = format!("{}/call", server.url.trim_end_matches('/'));
                let body = serde_json::json!({
                    "tool": request.tool,
                    "params": request.params,
                });
                let resp = self
                    .http_client
                    .post(&url)
                    .json(&body)
                    .send()
                    .map_err(|e| MCPError(format!("HTTP request failed: {}", e)))?;
                let data: MCPResponse = resp
                    .json()
                    .map_err(|e| MCPError(format!("JSON decode failed: {}", e)))?;
                return Ok(data);
            }
        }
        Err(MCPError(format!(
            "Tool '{}' not found on any MCP server",
            request.tool
        )))
    }

    /// Requests the tool list from an MCP server URL and returns decoded tool metadata.
    pub fn list_tools(&self, server_url: &str) -> Result<Vec<MCPTool>, MCPError> {
        let url = format!("{}/list_tools", server_url.trim_end_matches('/'));
        let resp = self
            .http_client
            .get(&url)
            .send()
            .map_err(|e| MCPError(format!("HTTP request failed: {}", e)))?;
        let tools: Vec<MCPTool> = resp
            .json()
            .map_err(|e| MCPError(format!("JSON decode failed: {}", e)))?;
        Ok(tools)
    }

    /// Registers or replaces an MCP server definition with its advertised tools.
    pub fn register_server(
        &mut self,
        name: &str,
        url: &str,
        tools: Vec<MCPTool>,
    ) -> Result<(), MCPError> {
        let server = MCPServer {
            name: name.to_string(),
            url: url.to_string(),
            tools: tools.clone(),
        };
        self.servers.retain(|s| s.name != name);
        self.servers.push(server);
        Ok(())
    }

    /// Exports registry capabilities as MCP tool metadata.
    pub fn export_registry_as_mcp(&self) -> Result<Vec<MCPTool>, MornError> {
        let registry = self
            .registry
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(registry
            .list_all()
            .iter()
            .map(|cap| MCPTool {
                name: cap.name.clone(),
                description: cap.description.clone(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": format!("One of: {}", cap.actions.join(", ")),
                        },
                    },
                }),
                server_url: None,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn test_mcp_export_registry() {
        let storage = Storage::new_in_memory().ok();
        let event_bus = crate::core::event_bus::SimpleEventBus::new();
        let registry = Arc::new(Mutex::new(Registry::new(storage, Some(event_bus))));
        let client = MCPClient::new(registry);
        let tools = client.export_registry_as_mcp().unwrap();
        assert!(!tools.is_empty(), "Should export at least one tool");
        assert!(tools.iter().any(|t| t.name == "Chat Agent"));
    }

    #[test]
    fn test_mcp_call_tool_not_found() {
        let storage = Storage::new_in_memory().ok();
        let event_bus = crate::core::event_bus::SimpleEventBus::new();
        let registry = Arc::new(Mutex::new(Registry::new(storage, Some(event_bus))));
        let client = MCPClient::new(registry);
        let req = MCPRequest {
            tool: "nonexistent".to_string(),
            params: serde_json::json!({}),
        };
        let result = client.call_tool(req);
        assert!(result.is_err());
    }
}
