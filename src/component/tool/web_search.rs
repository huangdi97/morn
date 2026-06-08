//! web_search — Provides a tool for web search requests.
use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置 Web 搜索工具注册入口 */
pub struct WebSearchTool {
    id: String,
    name: String,
}

impl WebSearchTool {
    pub fn new() -> Self {
        WebSearchTool {
            id: "tool-web-search".into(),
            name: "Web Search".into(),
        }
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for WebSearchTool {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "tool"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for WebSearchTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "search query".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "search results".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("WebSearchTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for WebSearchTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Tool for WebSearchTool {
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let query = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[web_search] simulated results for: {}",
            query
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_web_search_tool_has_expected_component_metadata() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.id(), "tool-web-search");
        assert_eq!(tool.type_name(), "tool");
        assert_eq!(tool.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn web_search_tool_exposes_text_input_and_output_ports() {
        let tool = WebSearchTool::new();
        let ports = tool.ports();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].id, "input");
        assert_eq!(ports[0].direction, PortDirection::Input);
        assert_eq!(ports[1].id, "output");
        assert_eq!(ports[1].direction, PortDirection::Output);
    }

    #[test]
    fn web_search_tool_requires_network_permission() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.required_permissions(), vec![Permission::NetworkAccess]);
    }

    #[test]
    fn execute_includes_query_in_simulated_results() {
        let mut tool = WebSearchTool::new();
        let result = tool.execute(Data::text("rust traits")).unwrap();
        let text = result.content.as_str().unwrap();
        assert!(text.contains("[web_search]"));
        assert!(text.contains("rust traits"));
    }
}
