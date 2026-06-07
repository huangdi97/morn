//! builtins — Registers built-in tools for search, files, and code execution.
use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置时间工具注册入口 */
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

#[allow(dead_code)] /* 预留：内置计算工具注册入口 */
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

#[allow(dead_code)] /* 预留：内置消息发送工具注册入口 */
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
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("SendMsgTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
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
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let msg = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!("[send_msg] sent: {}", msg)))
    }
}

#[allow(dead_code)] /* 预留：内置 HTTP 请求工具注册入口 */
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
    fn send(&mut self, port: &str, _data: Data) -> Result<(), String> {
        Err(format!("HttpRequestTool cannot receive on port {}", port))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String> {
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
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let url = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!("[http_request] GET {} - 200 OK", url)))
    }
}
