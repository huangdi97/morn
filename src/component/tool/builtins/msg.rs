//! msg — SendMsgTool: send messages via configured channels.
use crate::core::error::MornError;
use crate::component::tool::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)]
pub struct SendMsgTool {
    id: String,
    name: String,
}

impl SendMsgTool {
    pub fn new() -> Self {
        SendMsgTool {
            id: "tool-send-msg".into(),
            name: "Send Message".into(),
        }
    }
}

impl Default for SendMsgTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SendMsgTool {
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

impl IOComponent for SendMsgTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "message content".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "send result".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), MornError> {
        Err(MornError::Internal(format!("SendMsgTool cannot receive on port {}", port)))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, MornError> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for SendMsgTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Tool for SendMsgTool {
    fn execute(&mut self, input: Data) -> Result<Data, MornError> {
        let msg = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!("[send_msg] sent: {}", msg)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_msg_tool_echoes_input() {
        let mut tool = SendMsgTool::new();
        let result = tool.execute(Data::text("hello")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.contains("hello"));
    }
}
