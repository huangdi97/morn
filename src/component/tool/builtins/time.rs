//! time — GetTimeTool: retrieve current date and time.
use crate::component::tool::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
pub struct GetTimeTool {
    id: String,
    name: String,
}

impl GetTimeTool {
    pub fn new() -> Self {
        GetTimeTool {
            id: "tool-get-time".into(),
            name: "Get Time".into(),
        }
    }
}

impl Default for GetTimeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for GetTimeTool {
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

impl IOComponent for GetTimeTool {
    fn ports(&self) -> Vec<Port> {
        vec![Port {
            id: "output".into(),
            direction: PortDirection::Output,
            data_type: "text".into(),
            description: "current time".into(),
        }]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("GetTimeTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for GetTimeTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Tool for GetTimeTool {
    fn execute(&mut self, _input: Data) -> Result<Data, String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        Ok(Data::text(&now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_time_tool_returns_formatted_time() {
        let mut tool = GetTimeTool::new();
        let result = tool.execute(Data::text("")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.contains("UTC"));
        assert!(output.len() > 15);
    }
}
