use std::collections::HashMap;

use crate::core::mcp::{MCPError, MCPResponse, MCPTool};
use crate::mcp::http::call_http;
use crate::mcp::stdio::call_stdio;
use crate::mcp::server::{MCPServer, MCPTransport, ServerStatus};

#[derive(Debug, Clone)]
pub struct MCPManager {
    servers: HashMap<String, MCPServer>,
}

impl MCPManager {
    pub fn new() -> Self {
        MCPManager {
            servers: HashMap::new(),
        }
    }

    pub fn register_server(&mut self, server: MCPServer) {
        self.servers.insert(server.name.clone(), server);
    }

    pub fn discover_tools(&self) -> Vec<MCPTool> {
        self.servers
            .values()
            .flat_map(|s| s.tools.clone())
            .collect()
    }

    pub fn call_tool(&self, server_name: &str, tool_name: &str, args: &serde_json::Value) -> Result<MCPResponse, MCPError> {
        let server = self.servers.get(server_name).ok_or_else(|| {
            MCPError(format!("Server '{}' not found", server_name))
        })?;

        if !server.tools.iter().any(|t| t.name == tool_name) {
            return Err(MCPError(format!(
                "Tool '{}' not found on server '{}'",
                tool_name, server_name
            )));
        }

        match &server.transport {
            MCPTransport::Http { url, api_key } => call_http(url, api_key, tool_name, args),
            MCPTransport::Stdio { command, args: cmd_args } => {
                call_stdio(command, cmd_args, tool_name, args)
            }
        }
    }

    pub fn disconnect(&mut self, server_name: &str) {
        if let Some(server) = self.servers.get_mut(server_name) {
            server.status = ServerStatus::Disconnected;
        }
    }
}

impl Default for MCPManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stdio_server(name: &str) -> MCPServer {
        MCPServer {
            name: name.to_string(),
            transport: MCPTransport::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            tools: vec![MCPTool {
                name: "greet".to_string(),
                description: "greeting tool".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                server_url: None,
            }],
            status: ServerStatus::Connected,
        }
    }

    #[test]
    fn test_manager_new_empty() {
        let manager = MCPManager::new();
        assert_eq!(manager.discover_tools().len(), 0);
    }

    #[test]
    fn test_manager_register_and_discover() {
        let mut manager = MCPManager::new();
        manager.register_server(make_stdio_server("s1"));
        manager.register_server(make_stdio_server("s2"));
        let tools = manager.discover_tools();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_manager_call_tool_unknown_server() {
        let manager = MCPManager::new();
        let result = manager.call_tool("no-such", "greet", &serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("not found"));
    }

    #[test]
    fn test_manager_call_tool_unknown_tool() {
        let mut manager = MCPManager::new();
        manager.register_server(make_stdio_server("s1"));
        let result = manager.call_tool("s1", "no-such-tool", &serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("not found on server"));
    }

    #[test]
    fn test_manager_disconnect() {
        let mut manager = MCPManager::new();
        manager.register_server(make_stdio_server("s1"));
        manager.disconnect("s1");
        let tools = manager.discover_tools();
        // tools are still present (just status changed)
        assert_eq!(tools.len(), 1);
    }

    #[test]
    fn test_manager_default() {
        let manager = MCPManager::default();
        assert_eq!(manager.discover_tools().len(), 0);
    }
}
