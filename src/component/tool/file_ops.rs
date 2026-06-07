use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
pub struct ReadFileTool {
    id: String,
    name: String,
}

impl ReadFileTool {
    pub fn new() -> Self {
        ReadFileTool {
            id: "tool-read-file".into(),
            name: "Read File".into(),
        }
    }
}

impl Component for ReadFileTool {
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

impl IOComponent for ReadFileTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "file path".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "file contents".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("ReadFileTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for ReadFileTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Tool for ReadFileTool {
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let path = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!("[read_file] contents of {}", path)))
    }
}

#[allow(dead_code)]
pub struct WriteFileTool {
    id: String,
    name: String,
}

impl WriteFileTool {
    pub fn new() -> Self {
        WriteFileTool {
            id: "tool-write-file".into(),
            name: "Write File".into(),
        }
    }
}

impl Component for WriteFileTool {
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

impl IOComponent for WriteFileTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "path and content".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "write result".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("WriteFileTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for WriteFileTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::WriteFile]
    }
}

impl Tool for WriteFileTool {
    fn execute(&mut self, _input: Data) -> Result<Data, String> {
        Ok(Data::text("[write_file] written successfully"))
    }
}
