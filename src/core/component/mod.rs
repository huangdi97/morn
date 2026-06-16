//! component — Defines shared component traits, data values, and execution context.
use crate::core::error::MornError;
pub mod trait_def;
pub mod types;

pub use crate::core::component::types::{
    ComponentType, Data, HealthStatus, Permission, Port, PortDirection,
};

pub trait Component: Send {
    fn id(&self) -> &str;
    fn type_name(&self) -> &str;
    fn init(&mut self) -> Result<(), MornError>;
    fn run(&mut self) -> Result<(), MornError>;
    fn pause(&mut self) -> Result<(), MornError>;
    fn stop(&mut self) -> Result<(), MornError>;
    fn health_check(&self) -> HealthStatus;
}

pub trait IOComponent: Component {
    fn ports(&self) -> Vec<types::Port>;
    fn send(&mut self, port: &str, data: types::Data) -> Result<(), MornError>;
    fn recv(&mut self, port: &str) -> Result<Option<types::Data>, MornError>;
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

        fn init(&mut self) -> Result<(), MornError> {
            self.status = HealthStatus::Healthy;
            Ok(())
        }

        fn run(&mut self) -> Result<(), MornError> {
            self.running = true;
            Ok(())
        }

        fn pause(&mut self) -> Result<(), MornError> {
            self.running = false;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), MornError> {
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
