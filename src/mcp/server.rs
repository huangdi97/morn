use serde::{Deserialize, Serialize};
use crate::core::mcp::MCPTool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String, api_key: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Connected,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub name: String,
    pub transport: MCPTransport,
    pub tools: Vec<MCPTool>,
    pub status: ServerStatus,
}
