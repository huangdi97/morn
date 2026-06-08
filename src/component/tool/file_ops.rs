//! file_ops — Provides tools for reading, writing, and managing files.
use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置读文件工具注册入口 */
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

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
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

#[allow(dead_code)] /* 预留：内置写文件工具注册入口 */
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

impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_tool_has_expected_component_metadata() {
        let tool = ReadFileTool::new();
        assert_eq!(tool.id(), "tool-read-file");
        assert_eq!(tool.type_name(), "tool");
        assert_eq!(tool.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn write_file_tool_has_expected_component_metadata() {
        let tool = WriteFileTool::new();
        assert_eq!(tool.id(), "tool-write-file");
        assert_eq!(tool.type_name(), "tool");
        assert_eq!(tool.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn file_tools_expose_text_input_and_output_ports() {
        for ports in [ReadFileTool::new().ports(), WriteFileTool::new().ports()] {
            assert_eq!(ports.len(), 2);
            assert_eq!(ports[0].id, "input");
            assert_eq!(ports[0].direction, PortDirection::Input);
            assert_eq!(ports[1].id, "output");
            assert_eq!(ports[1].direction, PortDirection::Output);
        }
    }

    #[test]
    fn file_tools_require_file_permissions() {
        assert_eq!(
            ReadFileTool::new().required_permissions(),
            vec![Permission::ReadFile]
        );
        assert_eq!(
            WriteFileTool::new().required_permissions(),
            vec![Permission::WriteFile]
        );
    }

    #[test]
    fn file_tools_execute_with_simulated_results() {
        let mut read_tool = ReadFileTool::new();
        let read_result = read_tool.execute(Data::text("/tmp/example.txt")).unwrap();
        assert!(read_result
            .content
            .as_str()
            .unwrap()
            .contains("/tmp/example.txt"));

        let mut write_tool = WriteFileTool::new();
        let write_result = write_tool.execute(Data::text("ignored")).unwrap();
        assert_eq!(
            write_result.content.as_str().unwrap(),
            "[write_file] written successfully"
        );
    }
}
