//! calc — CalcTool: basic arithmetic evaluation.
use crate::component::tool::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
pub struct CalcTool {
    id: String,
    name: String,
}

impl CalcTool {
    pub fn new() -> Self {
        CalcTool {
            id: "tool-calc".into(),
            name: "Calculator".into(),
        }
    }
}

impl Default for CalcTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CalcTool {
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

impl IOComponent for CalcTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "math expression".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "result".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("CalcTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for CalcTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Tool for CalcTool {
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let expr = input.content.as_str().unwrap_or("0").to_string();
        Ok(Data::text(&format!("[calc] {} = (simulated)", expr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_tool_simulates_result() {
        let mut tool = CalcTool::new();
        let result = tool.execute(Data::text("2+2")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.contains("[calc]"));
        assert!(output.contains("2+2"));
    }

    #[test]
    fn calc_tool_empty_input_does_not_panic() {
        let mut tool = CalcTool::new();
        let result = tool.execute(Data::text("")).unwrap();
        assert!(result.content.as_str().unwrap_or("").contains("[calc]"));
    }
}
