//! MCP 适配器 — 提供组件端口与 MCP 工具之间的互相转换函数。
//! MCP adapter — conversion functions between component ports and MCP tools.

use crate::core::component::Port;
use crate::core::mcp::MCPTool;

/// 将组件端口列表转换为一个 MCP 工具定义。
/// `ports` 中的每个端口都会成为 MCP 工具 `input_schema` 中的一个属性，
/// 包含类型、描述和方向信息。
///
/// Convert a slice of component ports into an MCP tool definition.
/// Each port in `ports` becomes a property in the tool's `input_schema`,
/// including its type, description, and direction.
pub fn port_to_mcp_tool(name: &str, description: &str, ports: &[Port]) -> MCPTool {
    let mut properties = serde_json::Map::new();
    for port in ports {
        properties.insert(
            port.id.clone(),
            serde_json::json!({
                "type": port.data_type,
                "description": port.description,
                "direction": match port.direction {
                    crate::core::component::PortDirection::Input => "input",
                    crate::core::component::PortDirection::Output => "output",
                    crate::core::component::PortDirection::Bidirectional => "both",
                },
            }),
        );
    }

    MCPTool {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": properties,
        }),
        server_url: None,
    }
}

/// 从 MCP 工具的 input_schema 中提取组件端口列表。
/// 遍历 schema 中的 `properties`，将每个属性的类型、描述还原为 `Port`。
///
/// Extract a list of component ports from an MCP tool's input_schema.
/// Iterates over the `properties` in the schema and reconstructs each `Port`
/// with its type and description.
pub fn mcp_tool_to_ports(tool: &MCPTool) -> Vec<Port> {
    let mut ports = Vec::new();
    if let Some(props) = tool
        .input_schema
        .get("properties")
        .and_then(|p| p.as_object())
    {
        for (key, val) in props {
            let data_type = val
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("string")
                .to_string();
            let description = val
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            ports.push(Port {
                id: key.clone(),
                direction: crate::core::component::PortDirection::Input,
                data_type,
                description,
            });
        }
    }
    ports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{Port, PortDirection};

    #[test]
    fn test_port_to_mcp_tool() {
        let ports = vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "string".into(),
                description: "Search query".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "object".into(),
                description: "Search results".into(),
            },
        ];
        let tool = port_to_mcp_tool("web_search", "Search the web", &ports);
        assert_eq!(tool.name, "web_search");
        assert!(tool.input_schema["properties"]["query"]["type"].as_str() == Some("string"));
    }

    #[test]
    fn test_mcp_tool_to_ports() {
        let tool = MCPTool {
            name: "calc".to_string(),
            description: "Calculator".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number", "description": "First number"},
                    "b": {"type": "number", "description": "Second number"},
                }
            }),
            server_url: None,
        };
        let ports = mcp_tool_to_ports(&tool);
        assert_eq!(ports.len(), 2);
        assert!(ports.iter().any(|p| p.id == "a"));
    }

    #[test]
    fn test_port_to_mcp_roundtrip() {
        let original = vec![Port {
            id: "input".into(),
            direction: PortDirection::Input,
            data_type: "text".into(),
            description: "Input data".into(),
        }];
        let tool = port_to_mcp_tool("test", "Test tool", &original);
        let restored = mcp_tool_to_ports(&tool);
        assert_eq!(original.len(), restored.len());
        assert_eq!(original[0].id, restored[0].id);
    }

    #[test]
    fn test_port_to_mcp_with_description() {
        let ports = vec![Port {
            id: "code".into(),
            direction: PortDirection::Input,
            data_type: "string".into(),
            description: "Source code input".into(),
        }];
        let tool = port_to_mcp_tool("code_analyzer", "Analyzes source code", &ports);
        assert_eq!(tool.name, "code_analyzer");
        assert_eq!(tool.description, "Analyzes source code");
        assert_eq!(
            tool.input_schema["properties"]["code"]["description"],
            "Source code input"
        );
    }

    #[test]
    fn test_mcp_tool_to_ports_empty_properties() {
        let tool = MCPTool {
            name: "empty".to_string(),
            description: "No props".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            server_url: None,
        };
        let ports = mcp_tool_to_ports(&tool);
        assert!(ports.is_empty());
    }

    #[test]
    fn test_mcp_tool_to_ports_uses_default_type() {
        let tool = MCPTool {
            name: "test".to_string(),
            description: "Test".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": {"description": "some value"}
                }
            }),
            server_url: None,
        };
        let ports = mcp_tool_to_ports(&tool);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].data_type, "string");
    }
}
