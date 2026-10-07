//! Execution environment abstraction.
//!
//! Isolation is a capability requirement, not an implementation choice baked
//! into Morn. Providers may represent a local process, container, microVM,
//! full VM, remote sandbox or physical executor.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::{ExecutionClass, ExecutionGuarantee};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExecutionEnvironmentTag;
pub type ExecutionEnvironmentId = Id<ExecutionEnvironmentTag>;

/// Compatibility alias retained for the public runtime API. The canonical
/// execution topology vocabulary lives in morn-kernel.
pub type IsolationClass = ExecutionClass;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentSpec {
    pub minimum_isolation: IsolationClass,
    pub os: Option<String>,
    pub runtime: Option<String>,
    #[serde(default)]
    pub required_guarantees: Vec<ExecutionGuarantee>,
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
            required_guarantees: Vec::new(),
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
    #[serde(default)]
    pub guarantees: Vec<ExecutionGuarantee>,
    pub runtime_ref: String,
}

pub trait ExecutionEnvironmentProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_isolation_classes(&self) -> Vec<IsolationClass>;
    fn supports_isolation(&self, class: IsolationClass) -> bool {
        self.supported_isolation_classes().contains(&class)
    }
    fn supported_guarantees(&self, class: IsolationClass) -> Vec<ExecutionGuarantee>;
    fn provision(&mut self, spec: &ExecutionEnvironmentSpec) -> Result<ExecutionEnvironmentHandle>;
    fn release(&mut self, handle: &ExecutionEnvironmentHandle) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentOffer {
    pub provider: String,
    pub isolation: IsolationClass,
    pub guarantees: Vec<ExecutionGuarantee>,
    pub estimated_cost_micros: Option<u64>,
    pub stateful: bool,
    pub checkpoint_resume: bool,
}

impl ExecutionEnvironmentOffer {
    pub fn from_provider(
        provider: &dyn ExecutionEnvironmentProvider,
        estimated_cost_micros: Option<u64>,
    ) -> Vec<Self> {
        provider
            .supported_isolation_classes()
            .into_iter()
            .map(|isolation| {
                let guarantees = provider.supported_guarantees(isolation);
                Self {
                    provider: provider.provider_name().to_string(),
                    isolation,
                    stateful: guarantees.contains(&ExecutionGuarantee::StatefulExecution),
                    checkpoint_resume: guarantees.contains(&ExecutionGuarantee::CheckpointResume),
                    guarantees,
                    estimated_cost_micros,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentSelection {
    pub provider: String,
    pub isolation: IsolationClass,
    pub guarantees: Vec<ExecutionGuarantee>,
    pub estimated_cost_micros: Option<u64>,
    pub rationale: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ExecutionEnvironmentResolver;

impl ExecutionEnvironmentResolver {
    pub fn resolve(
        &self,
        spec: &ExecutionEnvironmentSpec,
        offers: &[ExecutionEnvironmentOffer],
    ) -> Option<ExecutionEnvironmentSelection> {
        let mut candidates: Vec<&ExecutionEnvironmentOffer> = offers
            .iter()
            .filter(|offer| offer.isolation.satisfies(spec.minimum_isolation))
            .filter(|offer| {
                spec.required_guarantees
                    .iter()
                    .all(|required| offer.guarantees.contains(required))
            })
            .collect();

        candidates.sort_by(|left, right| {
            let left_cost = left.estimated_cost_micros.unwrap_or(u64::MAX);
            let right_cost = right.estimated_cost_micros.unwrap_or(u64::MAX);
            left_cost
                .cmp(&right_cost)
                .then_with(|| {
                    containment_preference(left.isolation)
                        .cmp(&containment_preference(right.isolation))
                })
                .then_with(|| left.provider.cmp(&right.provider))
        });

        candidates
            .first()
            .map(|offer| ExecutionEnvironmentSelection {
                provider: offer.provider.clone(),
                isolation: offer.isolation,
                guarantees: offer.guarantees.clone(),
                estimated_cost_micros: offer.estimated_cost_micros,
                rationale: vec![
                    format!("satisfies execution class {}", spec.minimum_isolation.key()),
                    format!(
                        "satisfies {} required guarantee(s)",
                        spec.required_guarantees.len()
                    ),
                ],
            })
    }
}

fn containment_preference(class: IsolationClass) -> u8 {
    match class {
        IsolationClass::NoIsolation => 0,
        IsolationClass::Process => 1,
        IsolationClass::Container => 2,
        IsolationClass::MicroVm => 3,
        IsolationClass::FullVm => 4,
        IsolationClass::Remote => 10,
        IsolationClass::Physical => 11,
    }
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

    fn supported_guarantees(&self, class: IsolationClass) -> Vec<ExecutionGuarantee> {
        let mut guarantees = vec![
            ExecutionGuarantee::FilesystemReadPolicy,
            ExecutionGuarantee::FilesystemWritePolicy,
            ExecutionGuarantee::ResourceLimits,
            ExecutionGuarantee::SecretIndirection,
        ];
        if matches!(
            class,
            IsolationClass::Container | IsolationClass::MicroVm | IsolationClass::FullVm
        ) {
            guarantees.push(ExecutionGuarantee::ProcessBoundary);
            guarantees.push(ExecutionGuarantee::NetworkEgressPolicy);
        }
        if matches!(class, IsolationClass::MicroVm | IsolationClass::FullVm) {
            guarantees.push(ExecutionGuarantee::KernelBoundary);
        }
        guarantees
    }

    fn provision(&mut self, spec: &ExecutionEnvironmentSpec) -> Result<ExecutionEnvironmentHandle> {
        if !self.supports_isolation(spec.minimum_isolation) {
            return Err(Error::external(format!(
                "requested execution class {:?} is not supported by provider {}",
                spec.minimum_isolation,
                self.provider_name()
            )));
        }
        let provided_guarantees = self.supported_guarantees(spec.minimum_isolation);
        if let Some(missing) = spec
            .required_guarantees
            .iter()
            .find(|required| !provided_guarantees.contains(required))
        {
            return Err(Error::external(format!(
                "missing execution guarantee {} from provider {}",
                missing.key(),
                self.provider_name()
            )));
        }

        let id = ExecutionEnvironmentId::generate_with("env");
        self.active.push(id.clone());
        Ok(ExecutionEnvironmentHandle {
            id,
            provider: self.provider_name().to_string(),
            isolation: spec.minimum_isolation,
            guarantees: provided_guarantees,
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
    fn resolver_selects_environment_from_requirements_not_provider_name() {
        let offers = vec![
            ExecutionEnvironmentOffer {
                provider: "container-a".to_string(),
                isolation: IsolationClass::Container,
                guarantees: vec![
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
                estimated_cost_micros: Some(10),
                stateful: false,
                checkpoint_resume: false,
            },
            ExecutionEnvironmentOffer {
                provider: "microvm-b".to_string(),
                isolation: IsolationClass::MicroVm,
                guarantees: vec![
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                    ExecutionGuarantee::KernelBoundary,
                    ExecutionGuarantee::RuntimeAttestation,
                ],
                estimated_cost_micros: Some(30),
                stateful: false,
                checkpoint_resume: false,
            },
        ];
        let selected = ExecutionEnvironmentResolver
            .resolve(
                &ExecutionEnvironmentSpec {
                    minimum_isolation: IsolationClass::Container,
                    required_guarantees: vec![ExecutionGuarantee::RuntimeAttestation],
                    ..Default::default()
                },
                &offers,
            )
            .unwrap();
        assert_eq!(selected.provider, "microvm-b");
        assert_eq!(selected.isolation, IsolationClass::MicroVm);
    }

    #[test]
    fn remote_offer_cannot_satisfy_local_container_requirement_by_ordering() {
        let offers = vec![ExecutionEnvironmentOffer {
            provider: "remote".to_string(),
            isolation: IsolationClass::Remote,
            guarantees: vec![
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
            ],
            estimated_cost_micros: Some(1),
            stateful: true,
            checkpoint_resume: true,
        }];
        assert!(ExecutionEnvironmentResolver
            .resolve(
                &ExecutionEnvironmentSpec {
                    minimum_isolation: IsolationClass::Container,
                    ..Default::default()
                },
                &offers,
            )
            .is_none());
    }

    #[test]
    fn remote_and_physical_are_not_security_rank_shortcuts() {
        let provider = FixtureEnvironmentProvider::default();
        assert!(provider.supports_isolation(IsolationClass::Container));
        assert!(provider.supports_isolation(IsolationClass::MicroVm));
        assert!(!provider.supports_isolation(IsolationClass::Remote));
        assert!(!provider.supports_isolation(IsolationClass::Physical));
    }

    #[test]
    fn environment_provider_enforces_guarantee_vector() {
        let mut provider = FixtureEnvironmentProvider::default();
        let ok = provider
            .provision(&ExecutionEnvironmentSpec {
                minimum_isolation: IsolationClass::Container,
                required_guarantees: vec![
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
                ..Default::default()
            })
            .unwrap();
        assert!(ok
            .guarantees
            .contains(&ExecutionGuarantee::NetworkEgressPolicy));

        let blocked = provider.provision(&ExecutionEnvironmentSpec {
            minimum_isolation: IsolationClass::Container,
            required_guarantees: vec![ExecutionGuarantee::RuntimeAttestation],
            ..Default::default()
        });
        assert!(blocked.is_err());
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
