use crate::core::mcp::MCPTool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPTransport {
    Stdio {
        command: String,
        args: Vec<String>,
    },
    Http {
        url: String,
        api_key: Option<String>,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_new_stdio() {
        let server = MCPServer {
            name: "test-server".to_string(),
            transport: MCPTransport::Stdio {
                command: "python".to_string(),
                args: vec!["script.py".to_string()],
            },
            tools: vec![MCPTool {
                name: "echo".to_string(),
                description: "echo tool".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                server_url: None,
            }],
            status: ServerStatus::Connected,
        };
        assert_eq!(server.name, "test-server");
        assert_eq!(server.status, ServerStatus::Connected);
        assert_eq!(server.tools.len(), 1);
        assert_eq!(server.tools[0].name, "echo");
    }

    #[test]
    fn test_mcp_server_new_http() {
        let server = MCPServer {
            name: "http-server".to_string(),
            transport: MCPTransport::Http {
                url: "http://localhost:8080".to_string(),
                api_key: Some("key123".to_string()),
            },
            tools: vec![],
            status: ServerStatus::Disconnected,
        };
        assert_eq!(server.name, "http-server");
        assert_eq!(server.status, ServerStatus::Disconnected);
    }

    #[test]
    fn test_server_status_partial_eq() {
        assert_eq!(ServerStatus::Connected, ServerStatus::Connected);
        assert_ne!(ServerStatus::Connected, ServerStatus::Disconnected);
    }

    #[test]
    fn test_server_serialize_deserialize() {
        let server = MCPServer {
            name: "s".to_string(),
            transport: MCPTransport::Stdio {
                command: "cmd".to_string(),
                args: vec![],
            },
            tools: vec![],
            status: ServerStatus::Error("oops".to_string()),
        };
        let json = serde_json::to_string(&server).unwrap();
        let deserialized: MCPServer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "s");
        assert_eq!(deserialized.status, ServerStatus::Error("oops".to_string()));
    }
}
