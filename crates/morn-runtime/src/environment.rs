//! Execution environment abstraction.
//!
//! Isolation is a capability requirement, not an implementation choice baked
//! into Morn. Providers may represent a local process, container, microVM,
//! full VM, remote sandbox or physical executor.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExecutionEnvironmentTag;
pub type ExecutionEnvironmentId = Id<ExecutionEnvironmentTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum IsolationClass {
    Process,
    Container,
    MicroVm,
    FullVm,
    Remote,
    Physical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentSpec {
    pub minimum_isolation: IsolationClass,
    pub os: Option<String>,
    pub runtime: Option<String>,
    pub cpu_millis: Option<u64>,
    pub memory_mb: Option<u64>,
    pub gpu_count: Option<u32>,
    pub network_allowlist: Vec<String>,
    pub writable_paths: Vec<String>,
    pub secret_refs: Vec<String>,
    pub persistence_scope: String,
    pub timeout_ms: Option<u64>,
    pub side_effect_policy: String,
}

impl Default for ExecutionEnvironmentSpec {
    fn default() -> Self {
        Self {
            minimum_isolation: IsolationClass::Process,
            os: None,
            runtime: None,
            cpu_millis: None,
            memory_mb: None,
            gpu_count: None,
            network_allowlist: Vec::new(),
            writable_paths: Vec::new(),
            secret_refs: Vec::new(),
            persistence_scope: "attempt".to_string(),
            timeout_ms: None,
            side_effect_policy: "profile-governed".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentHandle {
    pub id: ExecutionEnvironmentId,
    pub provider: String,
    pub isolation: IsolationClass,
    pub runtime_ref: String,
}

pub trait ExecutionEnvironmentProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_isolation_classes(&self) -> Vec<IsolationClass>;
    fn supports_isolation(&self, class: IsolationClass) -> bool {
        self.supported_isolation_classes().contains(&class)
    }
    fn provision(&mut self, spec: &ExecutionEnvironmentSpec) -> Result<ExecutionEnvironmentHandle>;
    fn release(&mut self, handle: &ExecutionEnvironmentHandle) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct FixtureEnvironmentProvider {
    active: Vec<ExecutionEnvironmentId>,
}

impl ExecutionEnvironmentProvider for FixtureEnvironmentProvider {
    fn provider_name(&self) -> &str {
        "fixture-environment"
    }

    fn supported_isolation_classes(&self) -> Vec<IsolationClass> {
        vec![
            IsolationClass::Process,
            IsolationClass::Container,
            IsolationClass::MicroVm,
        ]
    }

    fn provision(&mut self, spec: &ExecutionEnvironmentSpec) -> Result<ExecutionEnvironmentHandle> {
        if !self.supports_isolation(spec.minimum_isolation) {
            return Err(Error::external(format!(
                "requested execution class {:?} is not supported by provider {}",
                spec.minimum_isolation,
                self.provider_name()
            )));
        }
        let id = ExecutionEnvironmentId::generate_with("env");
        self.active.push(id.clone());
        Ok(ExecutionEnvironmentHandle {
            id,
            provider: self.provider_name().to_string(),
            isolation: spec.minimum_isolation,
            runtime_ref: "fixture://local".to_string(),
        })
    }

    fn release(&mut self, handle: &ExecutionEnvironmentHandle) -> Result<()> {
        let before = self.active.len();
        self.active.retain(|id| id != &handle.id);
        if before == self.active.len() {
            return Err(Error::not_found(format!(
                "execution environment {}",
                handle.id
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_and_physical_are_not_security_rank_shortcuts() {
        let provider = FixtureEnvironmentProvider::default();
        assert!(provider.supports_isolation(IsolationClass::Container));
        assert!(provider.supports_isolation(IsolationClass::MicroVm));
        assert!(!provider.supports_isolation(IsolationClass::Remote));
        assert!(!provider.supports_isolation(IsolationClass::Physical));
    }

    #[test]
    fn environment_provider_enforces_minimum_isolation() {
        let mut provider = FixtureEnvironmentProvider::default();
        let ok = provider
            .provision(&ExecutionEnvironmentSpec {
                minimum_isolation: IsolationClass::Container,
                ..Default::default()
            })
            .unwrap();
        provider.release(&ok).unwrap();

        let blocked = provider.provision(&ExecutionEnvironmentSpec {
            minimum_isolation: IsolationClass::FullVm,
            ..Default::default()
        });
        assert!(blocked.is_err());
    }
}
