use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::core::mcp::{MCPError, MCPResponse, MCPTool};
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
            MCPTransport::Http { url, api_key } => self.call_http(url, api_key, tool_name, args),
            MCPTransport::Stdio { command, args: cmd_args } => {
                self.call_stdio(command, cmd_args, tool_name, args)
            }
        }
    }

    fn call_http(
        &self,
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
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().map_err(|e| MCPError(format!("HTTP request failed: {}", e)))?;
        let data: MCPResponse = resp.json().map_err(|e| MCPError(format!("JSON decode failed: {}", e)))?;
        Ok(data)
    }

    fn call_stdio(
        &self,
        command: &str,
        cmd_args: &[String],
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<MCPResponse, MCPError> {
        let input = serde_json::json!({
            "tool": tool_name,
            "params": args,
        });

        let output = Command::new(command)
            .args(cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    serde_json::to_writer(stdin, &input)?;
                }
                child.wait_with_output()
            })
            .map_err(|e| MCPError(format!("Stdio command failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MCPError(format!("Command exited with error: {}", stderr)));
        }

        let data: MCPResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| MCPError(format!("JSON parse failed: {}", e)))?;
        Ok(data)
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
