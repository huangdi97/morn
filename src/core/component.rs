use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded(msg) => write!(f, "degraded: {}", msg),
            HealthStatus::Unhealthy(msg) => write!(f, "unhealthy: {}", msg),
        }
    }
}

pub trait Component: Send {
    fn id(&self) -> &str;
    fn type_name(&self) -> &str;
    fn init(&mut self) -> Result<(), String>;
    fn run(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn health_check(&self) -> HealthStatus;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub id: String,
    pub direction: PortDirection,
    pub data_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub content: Value,
    pub mime_type: String,
}

impl Data {
    pub fn new(content: Value, mime_type: &str) -> Self {
        Data {
            content,
            mime_type: mime_type.to_string(),
        }
    }

    pub fn text(text: &str) -> Self {
        Data {
            content: Value::String(text.to_string()),
            mime_type: "text/plain".to_string(),
        }
    }

    pub fn json(value: Value) -> Self {
        Data {
            content: value,
            mime_type: "application/json".to_string(),
        }
    }
}

pub trait IOComponent: Component {
    fn ports(&self) -> Vec<Port>;
    fn send(&mut self, port: &str, data: Data) -> Result<(), String>;
    fn recv(&mut self, port: &str) -> Result<Option<Data>, String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    ReadFile,
    WriteFile,
    ExecuteShell,
    NetworkAccess,
    SystemConfig,
    FileDelete,
    ApplicationControl,
}

pub trait SecureComponent: Component {
    fn required_permissions(&self) -> Vec<Permission>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Tool,
    Knowledge,
    Skill,
    Persona,
    Memory,
    Model,
    Agent,
    Pipeline,
}

impl ComponentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::Tool => "tool",
            ComponentType::Knowledge => "knowledge",
            ComponentType::Skill => "skill",
            ComponentType::Persona => "persona",
            ComponentType::Memory => "memory",
            ComponentType::Model => "model",
            ComponentType::Agent => "agent",
            ComponentType::Pipeline => "pipeline",
        }
    }
}
