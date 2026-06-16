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
