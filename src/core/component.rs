//! component — Defines shared component traits, data values, and execution context.
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        status: HealthStatus,
        running: bool,
    }

    impl Component for TestComponent {
        fn id(&self) -> &str {
            "test-component"
        }

        fn type_name(&self) -> &str {
            "test"
        }

        fn init(&mut self) -> Result<(), String> {
            self.status = HealthStatus::Healthy;
            Ok(())
        }

        fn run(&mut self) -> Result<(), String> {
            self.running = true;
            Ok(())
        }

        fn pause(&mut self) -> Result<(), String> {
            self.running = false;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), String> {
            self.running = false;
            Ok(())
        }

        fn health_check(&self) -> HealthStatus {
            self.status.clone()
        }
    }

    #[test]
    fn component_lifecycle_updates_state() {
        let mut component = TestComponent {
            status: HealthStatus::Degraded("booting".into()),
            running: false,
        };
        assert!(component.init().is_ok());
        assert!(component.run().is_ok());
        assert!(component.running);
        assert!(component.pause().is_ok());
        assert!(!component.running);
    }

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
        assert_eq!(Data::json(serde_json::json!({"ok": true})).mime_type, "application/json");
    }

    #[test]
    fn component_type_names_are_lowercase() {
        assert_eq!(ComponentType::Tool.as_str(), "tool");
        assert_eq!(ComponentType::Pipeline.as_str(), "pipeline");
    }
}
