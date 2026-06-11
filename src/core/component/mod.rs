//! component — Defines shared component traits, data values, and execution context.
pub mod trait_def;
pub mod types;

pub use crate::core::component::types::{
    ComponentType, Data, HealthStatus, Permission, Port, PortDirection,
};

pub trait Component: Send {
    fn id(&self) -> &str;
    fn type_name(&self) -> &str;
    fn init(&mut self) -> Result<(), String>;
    fn run(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn health_check(&self) -> HealthStatus;
}

pub trait IOComponent: Component {
    fn ports(&self) -> Vec<types::Port>;
    fn send(&mut self, port: &str, data: types::Data) -> Result<(), String>;
    fn recv(&mut self, port: &str) -> Result<Option<types::Data>, String>;
}

pub trait SecureComponent: Component {
    fn required_permissions(&self) -> Vec<Permission>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::types::HealthStatus;

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
}
