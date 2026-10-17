//! Capability Fabric: what a capability can do, who/what provides it, and how
//! it is invoked with health, timeout, fallback and context. No AI-only
//! assumptions: Rule/Program/Solver/Model/LLM/Tool/API/Service/Human/Device/
//! Hybrid are all equal provider kinds.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::CapabilityProviderId;
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::definition::CapabilityDefinition;

/// Provider kind (provider-neutral).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CapabilityKind {
    Rule,
    Program,
    Solver,
    Model,
    Llm,
    Agent,
    Tool,
    Api,
    Service,
    Human,
    Device,
    Hybrid,
}

impl CapabilityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapabilityKind::Rule => "rule",
            CapabilityKind::Program => "program",
            CapabilityKind::Solver => "solver",
            CapabilityKind::Model => "model",
            CapabilityKind::Llm => "llm",
            CapabilityKind::Agent => "agent",
            CapabilityKind::Tool => "tool",
            CapabilityKind::Api => "api",
            CapabilityKind::Service => "service",
            CapabilityKind::Human => "human",
            CapabilityKind::Device => "device",
            CapabilityKind::Hybrid => "hybrid",
        }
    }
}

/// A registered capability provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityBinding {
    pub id: CapabilityProviderId,
    pub definition: CapabilityDefinition,
    pub kind: CapabilityKind,
    pub version: Version,
    pub health: bool,
    pub status: String,
    pub created_at: Timestamp,
}

/// Result of an invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationResult {
    pub ok: bool,
    pub output: String,
    pub latency_ms: u64,
    pub cost: f64,
    pub error: Option<String>,
}

/// A capability registry with invoke/health/fallback.
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    pub bindings: Vec<CapabilityBinding>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        definition: CapabilityDefinition,
        kind: CapabilityKind,
    ) -> CapabilityBinding {
        let binding = CapabilityBinding {
            id: CapabilityProviderId::generate_with("cprov"),
            definition,
            kind,
            version: Version::v1(),
            health: true,
            status: "registered".to_string(),
            created_at: Timestamp::now(),
        };
        self.bindings.push(binding.clone());
        binding
    }

    /// Invoke a capability by name. Deterministic kinds execute via the closure;
    /// others are exercised by their provider fixture (here simulated).
    pub fn invoke(
        &self,
        name: &str,
        input: &str,
        executor: impl FnOnce(&str) -> InvocationResult,
    ) -> Result<InvocationResult> {
        let binding = self
            .bindings
            .iter()
            .find(|b| b.definition.name == name)
            .ok_or_else(|| Error::not_found(format!("capability {name}")))?;
        if !binding.health {
            return Err(Error::external(format!("capability {name} is unhealthy")));
        }
        Ok(executor(input))
    }

    /// Fallback: try primary, then fallback capability.
    pub fn invoke_with_fallback(
        &self,
        primary: &str,
        fallback: &str,
        input: &str,
        primary_exec: impl FnOnce(&str) -> InvocationResult,
        fallback_exec: impl FnOnce(&str) -> InvocationResult,
    ) -> Result<InvocationResult> {
        match self.invoke(primary, input, primary_exec) {
            Ok(r) if r.ok => Ok(r),
            Ok(_) | Err(_) => {
                let binding = self
                    .bindings
                    .iter()
                    .find(|b| b.definition.name == fallback)
                    .ok_or_else(|| Error::not_found(format!("fallback capability {fallback}")))?;
                if !binding.health {
                    return Err(Error::external(format!("fallback {fallback} unhealthy")));
                }
                Ok(fallback_exec(input))
            }
        }
    }

    pub fn set_health(&mut self, id: &CapabilityProviderId, healthy: bool) {
        if let Some(b) = self.bindings.iter_mut().find(|b| b.id == *id) {
            b.health = healthy;
            b.status = if healthy { "healthy" } else { "unhealthy" }.to_string();
        }
    }
}

/// Convenience executor kinds.
pub fn rule_result(ok: bool, output: &str) -> InvocationResult {
    InvocationResult {
        ok,
        output: output.to_string(),
        latency_ms: 1,
        cost: 0.0,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(name: &str) -> CapabilityDefinition {
        CapabilityDefinition::new(name, "cap", vec![], "E0")
    }

    #[test]
    fn register_invoke_and_health() {
        let mut reg = CapabilityRegistry::new();
        let b = reg.register(def("qc-rule"), CapabilityKind::Rule);
        reg.register(def("qc-solver"), CapabilityKind::Solver);
        let r = reg
            .invoke("qc-rule", "rows=4", |i| {
                rule_result(i.contains("4"), "pass")
            })
            .unwrap();
        assert!(r.ok);
        reg.set_health(&b.id, false);
        assert!(reg
            .invoke("qc-rule", "x", |_| rule_result(true, "x"))
            .is_err());
    }

    #[test]
    fn fallback_is_used_when_primary_fails() {
        let mut reg = CapabilityRegistry::new();
        reg.register(def("primary"), CapabilityKind::Llm);
        reg.register(def("fallback"), CapabilityKind::Program);
        let r = reg
            .invoke_with_fallback(
                "primary",
                "fallback",
                "x",
                |_| rule_result(false, "primary failed"),
                |i| rule_result(true, &format!("fallback handled {i}")),
            )
            .unwrap();
        assert!(r.ok);
        assert!(r.output.contains("fallback"));
    }

    #[test]
    fn all_kinds_are_first_class() {
        let mut reg = CapabilityRegistry::new();
        for kind in [
            CapabilityKind::Rule,
            CapabilityKind::Program,
            CapabilityKind::Solver,
            CapabilityKind::Model,
            CapabilityKind::Llm,
            CapabilityKind::Agent,
            CapabilityKind::Tool,
            CapabilityKind::Api,
            CapabilityKind::Service,
            CapabilityKind::Human,
            CapabilityKind::Device,
            CapabilityKind::Hybrid,
        ] {
            reg.register(def(kind.as_str()), kind);
        }
        assert_eq!(reg.bindings.len(), 12);
        assert!(reg.bindings.iter().any(|b| b.kind == CapabilityKind::Human));
    }
}
