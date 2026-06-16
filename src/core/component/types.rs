//! types — Type definitions for the Component system.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_display_is_stable() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(
            HealthStatus::Unhealthy("down".into()).to_string(),
            "unhealthy: down"
        );
    }

    #[test]
    fn data_constructors_set_mime_types() {
        assert_eq!(Data::text("hello").mime_type, "text/plain");
        assert_eq!(
            Data::json(serde_json::json!({"ok": true})).mime_type,
            "application/json"
        );
    }

    #[test]
    fn component_type_names_are_lowercase() {
        assert_eq!(ComponentType::Tool.as_str(), "tool");
        assert_eq!(ComponentType::Pipeline.as_str(), "pipeline");
    }
}
