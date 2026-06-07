use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
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
