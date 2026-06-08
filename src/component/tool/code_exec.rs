//! code_exec — Provides a tool for running code execution tasks.
use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置 Python 执行工具注册入口 */
pub struct ExecPythonTool {
    id: String,
    name: String,
}

impl ExecPythonTool {
    pub fn new() -> Self {
        ExecPythonTool {
            id: "tool-exec-python".into(),
            name: "Exec Python".into(),
        }
    }
}

impl Default for ExecPythonTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ExecPythonTool {
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

impl IOComponent for ExecPythonTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "python script".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "execution result".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("ExecPythonTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for ExecPythonTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ExecuteShell]
    }
}

impl Tool for ExecPythonTool {
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let script = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[exec_python] executed: {} lines",
            script.lines().count()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_exec_python_tool_has_expected_component_metadata() {
        let tool = ExecPythonTool::new();
        assert_eq!(tool.id(), "tool-exec-python");
        assert_eq!(tool.type_name(), "tool");
        assert_eq!(tool.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn exec_python_tool_exposes_text_input_and_output_ports() {
        let tool = ExecPythonTool::new();
        let ports = tool.ports();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].id, "input");
        assert_eq!(ports[0].direction, PortDirection::Input);
        assert_eq!(ports[1].id, "output");
        assert_eq!(ports[1].direction, PortDirection::Output);
    }

    #[test]
    fn exec_python_tool_requires_shell_permission() {
        let tool = ExecPythonTool::new();
        assert_eq!(tool.required_permissions(), vec![Permission::ExecuteShell]);
    }

    #[test]
    fn execute_reports_script_line_count() {
        let mut tool = ExecPythonTool::new();
        let result = tool.execute(Data::text("print(1)\nprint(2)")).unwrap();
        assert_eq!(
            result.content.as_str().unwrap(),
            "[exec_python] executed: 2 lines"
        );
    }
}
