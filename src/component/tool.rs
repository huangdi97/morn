use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

pub trait Tool: IOComponent {
    fn execute(&mut self, input: Data) -> Result<Data, String>;
}

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

#[allow(dead_code)]
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

pub fn create_default_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(WebSearchTool::new()),
        Box::new(ReadFileTool::new()),
        Box::new(WriteFileTool::new()),
        Box::new(ExecPythonTool::new()),
        Box::new(GetTimeTool::new()),
        Box::new(CalcTool::new()),
        Box::new(SendMsgTool::new()),
        Box::new(HttpRequestTool::new()),
    ]
}

pub fn get_tool_by_name(name: &str) -> Option<Box<dyn Tool>> {
    match name {
        "web_search" => Some(Box::new(WebSearchTool::new())),
        "read_file" => Some(Box::new(ReadFileTool::new())),
        "write_file" => Some(Box::new(WriteFileTool::new())),
        "exec_python" => Some(Box::new(ExecPythonTool::new())),
        "get_time" => Some(Box::new(GetTimeTool::new())),
        "calc" => Some(Box::new(CalcTool::new())),
        "send_msg" => Some(Box::new(SendMsgTool::new())),
        "http_request" => Some(Box::new(HttpRequestTool::new())),
        _ => None,
    }
}
