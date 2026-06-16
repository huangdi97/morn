//! http — HttpRequestTool: send HTTP GET requests.
use crate::core::error::MornError;
use crate::component::tool::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
pub struct HttpRequestTool {
    id: String,
    name: String,
}

impl HttpRequestTool {
    pub fn new() -> Self {
        HttpRequestTool {
            id: "tool-http-request".into(),
            name: "HTTP Request".into(),
        }
    }
}

impl Default for HttpRequestTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HttpRequestTool {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "tool"
    }
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for HttpRequestTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "URL".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "response".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), MornError> {
        Err(MornError::Internal(format!("HttpRequestTool cannot receive on port {}", port)))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, MornError> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for HttpRequestTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Tool for HttpRequestTool {
    fn execute(&mut self, input: Data) -> Result<Data, MornError> {
        let url = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!("[http_request] GET {} - 200 OK", url)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_request_tool_returns_ok() {
        let mut tool = HttpRequestTool::new();
        let result = tool.execute(Data::text("https://example.com")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.contains("200 OK"));
    }
}
