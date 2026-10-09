//! Execution environment abstraction.
//!
//! Isolation is a capability requirement, not an implementation choice baked
//! into Morn. Providers may represent a local process, container, microVM,
//! full VM, remote sandbox or physical executor.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentAttestation {
    pub environment_ref: String,
    pub provider: String,
    pub isolation: IsolationClass,
    /// Attested capabilities/limits of this exact environment. Its
    /// minimum_isolation must equal `isolation`.
    pub attested_spec: ExecutionEnvironmentSpec,
    /// Exact provider runtime artifacts independently attested as present in
    /// this environment. These are distribution identities, not generic
    /// runtime kinds: `provider@version#sha256:<64-hex>`.
    #[serde(default)]
    pub runtime_identities: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub valid_until: Timestamp,
}

impl ExecutionEnvironmentAttestation {
    pub fn validate(&self) -> Result<()> {
        if self.environment_ref.trim().is_empty() || self.provider.trim().is_empty() {
            return Err(Error::validation(
                "execution environment attestation requires environment and provider identity",
            ));
        }
        if self.attested_spec.minimum_isolation != self.isolation {
            return Err(Error::validation(
                "attested execution spec isolation must match the environment isolation",
            ));
        }
        for identity in &self.runtime_identities {
            validate_runtime_identity(identity)?;
        }
        if self.evidence_refs.is_empty() {
            return Err(Error::validation(
                "execution environment attestation requires evidence",
            ));
        }
        if self.valid_until < self.observed_at {
            return Err(Error::validation(
                "execution environment attestation validity cannot precede observation",
            ));
        }
        Ok(())
    }

    pub fn active_at(&self, now: Timestamp) -> bool {
        self.validate().is_ok() && self.observed_at <= now && now <= self.valid_until
    }

    /// A live provider may be selected only when the deployment attestation
    /// independently binds this exact environment to the exact distribution
    /// identity that will be pinned into ExecutionBinding.
    pub fn attests_runtime_identity(
        &self,
        provider: &str,
        version: &str,
        digest: &str,
        now: Timestamp,
    ) -> bool {
        let Ok(identity) = exact_runtime_identity(provider, version, digest) else {
            return false;
        };
        self.active_at(now) && self.runtime_identities.iter().any(|item| item == &identity)
    }

    pub fn satisfies(&self, requested: &ExecutionEnvironmentSpec, now: Timestamp) -> bool {
        self.active_at(now)
            && self.isolation.satisfies(requested.minimum_isolation)
            && requested
                .required_guarantees
                .iter()
                .all(|required| self.attested_spec.required_guarantees.contains(required))
            && option_matches(&requested.os, &self.attested_spec.os)
            && option_matches(&requested.runtime, &self.attested_spec.runtime)
            && capacity_satisfies(requested.cpu_millis, self.attested_spec.cpu_millis)
            && capacity_satisfies(requested.memory_mb, self.attested_spec.memory_mb)
            && capacity_satisfies(requested.gpu_count, self.attested_spec.gpu_count)
            && requested
                .network_allowlist
                .iter()
                .all(|item| self.attested_spec.network_allowlist.contains(item))
            && requested
                .writable_paths
                .iter()
                .all(|item| self.attested_spec.writable_paths.contains(item))
            && requested
                .secret_refs
                .iter()
                .all(|item| self.attested_spec.secret_refs.contains(item))
            && self.attested_spec.persistence_scope == requested.persistence_scope
            && self.attested_spec.side_effect_policy == requested.side_effect_policy
            && requested.timeout_ms.is_none_or(|required| {
                self.attested_spec
                    .timeout_ms
                    .is_some_and(|actual| actual >= required)
            })
    }
}

pub fn exact_runtime_identity(provider: &str, version: &str, digest: &str) -> Result<String> {
    if provider.trim().is_empty() || version.trim().is_empty() {
        return Err(Error::validation(
            "runtime identity requires non-empty provider and version",
        ));
    }
    validate_sha256_digest(digest)?;
    Ok(format!("{provider}@{version}#{digest}"))
}

fn validate_runtime_identity(identity: &str) -> Result<()> {
    let (provider_version, digest) = identity.rsplit_once('#').ok_or_else(|| {
        Error::validation(
            "runtime identity must use provider@version#sha256:<64-hex>",
        )
    })?;
    let (provider, version) = provider_version.rsplit_once('@').ok_or_else(|| {
        Error::validation(
            "runtime identity must use provider@version#sha256:<64-hex>",
        )
    })?;
    let expected = exact_runtime_identity(provider, version, digest)?;
    if expected != identity {
        return Err(Error::validation(
            "runtime identity must use provider@version#sha256:<64-hex>",
        ));
    }
    Ok(())
}

fn validate_sha256_digest(digest: &str) -> Result<()> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(Error::validation(
            "runtime identity digest must use sha256:<64-hex>",
        ));
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::validation(
            "runtime identity digest must use sha256:<64-hex>",
        ));
    }
    Ok(())
}

fn option_matches(requested: &Option<String>, actual: &Option<String>) -> bool {
    requested.as_ref().is_none_or(|required| {
        actual
            .as_ref()
            .is_some_and(|value| value.eq_ignore_ascii_case(required))
    })
}

fn capacity_satisfies<T: PartialOrd + Copy>(requested: Option<T>, actual: Option<T>) -> bool {
    requested.is_none_or(|required| actual.is_some_and(|value| value >= required))
}

/// Adapter for environments that are provisioned/attested by an external
/// platform (Kubernetes, container service, microVM fleet, customer sandbox,
/// etc.). Morn leases an already-attested environment; it does not pretend to
/// have created or contained the runtime itself.
#[derive(Debug)]
pub struct AttestedExecutionEnvironmentProvider {
    name: String,
    attestations: BTreeMap<String, ExecutionEnvironmentAttestation>,
    leases: BTreeMap<String, String>,
}

impl AttestedExecutionEnvironmentProvider {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(Error::validation(
                "attested execution environment provider name is required",
            ));
        }
        Ok(Self {
            name,
            attestations: BTreeMap::new(),
            leases: BTreeMap::new(),
        })
    }

    pub fn register_attestation(
        &mut self,
        mut attestation: ExecutionEnvironmentAttestation,
    ) -> Result<()> {
        attestation.validate()?;
        if attestation.provider != self.name {
            return Err(Error::validation(
                "execution environment attestation belongs to another provider",
            ));
        }
        attestation.attested_spec.required_guarantees.sort();
        attestation.attested_spec.required_guarantees.dedup();
        attestation.runtime_identities.sort();
        attestation.runtime_identities.dedup();
        if let Some(existing) = self.attestations.get(&attestation.environment_ref) {
            if attestation.observed_at < existing.observed_at {
                return Err(Error::conflict(
                    "stale execution environment attestation cannot replace newer evidence",
                ));
            }
            if attestation.observed_at == existing.observed_at {
                if &attestation == existing {
                    return Ok(());
                }
                return Err(Error::conflict(
                    "conflicting execution environment attestations share the same observation time",
                ));
            }
        }
        self.attestations
            .insert(attestation.environment_ref.clone(), attestation);
        Ok(())
    }

    pub fn attestation(&self, environment_ref: &str) -> Option<&ExecutionEnvironmentAttestation> {
        self.attestations.get(environment_ref)
    }

    /// Deployment-safe discovery surface. Attestations expose only runtime
    /// identity, guarantee metadata and evidence references; they contain no secrets.
    pub fn attestations(&self) -> Vec<ExecutionEnvironmentAttestation> {
        self.attestations.values().cloned().collect()
    }

    fn environment_is_leased(&self, environment_ref: &str) -> bool {
        self.leases.values().any(|leased| leased == environment_ref)
    }
}

impl ExecutionEnvironmentProvider for AttestedExecutionEnvironmentProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn supported_isolation_classes(&self) -> Vec<IsolationClass> {
        let now = Timestamp::now();
        let mut classes = Vec::new();
        for attestation in self
            .attestations
            .values()
            .filter(|attestation| attestation.active_at(now))
        {
            if !classes.contains(&attestation.isolation) {
                classes.push(attestation.isolation);
            }
        }
        classes.sort_by_key(|class| containment_preference(*class));
        classes
    }

    fn supported_guarantees(&self, class: IsolationClass) -> Vec<ExecutionGuarantee> {
        let now = Timestamp::now();
        let mut active = self
            .attestations
            .values()
            .filter(|attestation| attestation.isolation == class && attestation.active_at(now));
        let Some(first) = active.next() else {
            return Vec::new();
        };
        // Intersection is deliberately conservative. Union would falsely claim
        // that one environment can combine guarantees present on different
        // machines/sandboxes.
        let mut common: BTreeSet<ExecutionGuarantee> = first
            .attested_spec
            .required_guarantees
            .iter()
            .copied()
            .collect();
        for attestation in active {
            let guarantees: BTreeSet<_> = attestation
                .attested_spec
                .required_guarantees
                .iter()
                .copied()
                .collect();
            common = common.intersection(&guarantees).copied().collect();
        }
        common.into_iter().collect()
    }

    fn provision(&mut self, spec: &ExecutionEnvironmentSpec) -> Result<ExecutionEnvironmentHandle> {
        let now = Timestamp::now();
        let selected = self
            .attestations
            .values()
            .filter(|attestation| !self.environment_is_leased(&attestation.environment_ref))
            .filter(|attestation| attestation.satisfies(spec, now))
            .min_by_key(|attestation| {
                (
                    containment_preference(attestation.isolation),
                    attestation.environment_ref.clone(),
                )
            })
            .cloned()
            .ok_or_else(|| {
                Error::external(
                    "no fresh attested execution environment satisfies the requested contract",
                )
            })?;

        let id = ExecutionEnvironmentId::generate_with("env-lease");
        self.leases
            .insert(id.to_string(), selected.environment_ref.clone());
        Ok(ExecutionEnvironmentHandle {
            id,
            provider: self.name.clone(),
            isolation: selected.isolation,
            guarantees: selected.attested_spec.required_guarantees.clone(),
            runtime_ref: selected.environment_ref,
        })
    }

    fn release(&mut self, handle: &ExecutionEnvironmentHandle) -> Result<()> {
        let environment_ref = self.leases.get(handle.id.as_str()).ok_or_else(|| {
            Error::not_found(format!("execution environment lease {}", handle.id))
        })?;
        if handle.provider != self.name || handle.runtime_ref != *environment_ref {
            return Err(Error::validation(
                "execution environment handle does not match its attested lease",
            ));
        }
        self.leases.remove(handle.id.as_str());
        Ok(())
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

    fn attestation(
        provider: &str,
        environment_ref: &str,
        guarantees: Vec<ExecutionGuarantee>,
    ) -> ExecutionEnvironmentAttestation {
        let now = Timestamp::now();
        ExecutionEnvironmentAttestation {
            environment_ref: environment_ref.to_string(),
            provider: provider.to_string(),
            isolation: IsolationClass::Container,
            attested_spec: ExecutionEnvironmentSpec {
                minimum_isolation: IsolationClass::Container,
                runtime: Some("oci".to_string()),
                required_guarantees: guarantees,
                network_allowlist: vec!["api.deepseek.com".to_string()],
                writable_paths: vec!["/workspace".to_string()],
                secret_refs: vec!["secret://deepseek".to_string()],
                ..Default::default()
            },
            runtime_identities: Vec::new(),
            evidence_refs: vec![format!("attestation://{environment_ref}")],
            observed_at: now,
            valid_until: Timestamp::from_millis(now.millis() + 60_000),
        }
    }

    #[test]
    fn runtime_identity_attestation_is_exact_and_time_scoped() {
        let now = Timestamp::from_millis(100);
        let digest = format!("sha256:{}", "a".repeat(64));
        let identity = exact_runtime_identity("deepseek-harness", "1.2.3", &digest).unwrap();
        let mut evidence = attestation(
            "sandbox-fleet",
            "env://sandbox/a",
            vec![ExecutionGuarantee::ProcessBoundary],
        );
        evidence.observed_at = Timestamp::from_millis(50);
        evidence.valid_until = Timestamp::from_millis(150);
        evidence.runtime_identities = vec![identity.clone()];
        assert!(evidence.validate().is_ok());
        assert!(evidence.attests_runtime_identity(
            "deepseek-harness",
            "1.2.3",
            &digest,
            now,
        ));
        assert!(!evidence.attests_runtime_identity(
            "deepseek-harness",
            "1.2.4",
            &digest,
            now,
        ));
        assert!(!evidence.attests_runtime_identity(
            "deepseek-harness",
            "1.2.3",
            &digest,
            Timestamp::from_millis(151),
        ));

        evidence.runtime_identities = vec!["deepseek-harness@1.2.3#sha256:not-a-digest".to_string()];
        assert!(evidence.validate().is_err());
    }

    #[test]
    fn attested_provider_discovery_starts_empty_without_deployment_evidence() {
        let provider = AttestedExecutionEnvironmentProvider::new("deployment-attestor").unwrap();
        assert!(provider.attestations().is_empty());
    }

    #[test]
    fn environment_attestation_updates_are_monotonic_and_replay_safe() {
        let mut provider = AttestedExecutionEnvironmentProvider::new("sandbox-fleet").unwrap();
        let mut current = attestation(
            "sandbox-fleet",
            "env://sandbox/a",
            vec![ExecutionGuarantee::ProcessBoundary],
        );
        current.observed_at = Timestamp::from_millis(100);
        current.valid_until = Timestamp::from_millis(1_000);
        provider.register_attestation(current.clone()).unwrap();

        // Exact redelivery is idempotent.
        provider.register_attestation(current.clone()).unwrap();

        let mut stale = current.clone();
        stale.observed_at = Timestamp::from_millis(99);
        assert!(provider.register_attestation(stale).is_err());

        let mut conflicting = current.clone();
        conflicting
            .attested_spec
            .required_guarantees
            .push(ExecutionGuarantee::NetworkEgressPolicy);
        assert!(provider.register_attestation(conflicting).is_err());

        let mut refreshed = current.clone();
        refreshed.observed_at = Timestamp::from_millis(101);
        refreshed.valid_until = Timestamp::from_millis(2_000);
        refreshed
            .attested_spec
            .required_guarantees
            .push(ExecutionGuarantee::NetworkEgressPolicy);
        provider.register_attestation(refreshed.clone()).unwrap();
        assert_eq!(
            provider.attestation("env://sandbox/a").unwrap().observed_at,
            refreshed.observed_at
        );
    }

    #[test]
    fn attested_provider_binds_one_exact_fresh_environment() {
        let mut provider = AttestedExecutionEnvironmentProvider::new("sandbox-fleet").unwrap();
        provider
            .register_attestation(attestation(
                "sandbox-fleet",
                "env://sandbox/a",
                vec![
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
            ))
            .unwrap();

        let requested = ExecutionEnvironmentSpec {
            minimum_isolation: IsolationClass::Container,
            runtime: Some("oci".to_string()),
            required_guarantees: vec![
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
            ],
            network_allowlist: vec!["api.deepseek.com".to_string()],
            writable_paths: vec!["/workspace".to_string()],
            secret_refs: vec!["secret://deepseek".to_string()],
            ..Default::default()
        };
        let handle = provider.provision(&requested).unwrap();
        assert_eq!(handle.runtime_ref, "env://sandbox/a");
        assert!(provider.provision(&requested).is_err());
        provider.release(&handle).unwrap();
        assert!(provider.provision(&requested).is_ok());
    }

    #[test]
    fn attested_provider_never_unions_guarantees_from_different_environments() {
        let mut provider = AttestedExecutionEnvironmentProvider::new("sandbox-fleet").unwrap();
        provider
            .register_attestation(attestation(
                "sandbox-fleet",
                "env://sandbox/network",
                vec![ExecutionGuarantee::NetworkEgressPolicy],
            ))
            .unwrap();
        provider
            .register_attestation(attestation(
                "sandbox-fleet",
                "env://sandbox/secret",
                vec![ExecutionGuarantee::SecretIndirection],
            ))
            .unwrap();

        assert!(provider
            .supported_guarantees(IsolationClass::Container)
            .is_empty());
        assert!(provider
            .provision(&ExecutionEnvironmentSpec {
                minimum_isolation: IsolationClass::Container,
                required_guarantees: vec![
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
                ..Default::default()
            })
            .is_err());
    }

    #[test]
    fn expired_or_cross_provider_environment_attestation_fails_closed() {
        let mut provider = AttestedExecutionEnvironmentProvider::new("sandbox-fleet").unwrap();
        let mut foreign = attestation(
            "other-provider",
            "env://other/a",
            vec![ExecutionGuarantee::ProcessBoundary],
        );
        assert!(provider.register_attestation(foreign.clone()).is_err());

        foreign.provider = "sandbox-fleet".to_string();
        foreign.valid_until = Timestamp::from_millis(foreign.observed_at.millis() - 1);
        assert!(provider.register_attestation(foreign.clone()).is_err());

        let mut expired = attestation(
            "sandbox-fleet",
            "env://sandbox/expired",
            vec![ExecutionGuarantee::ProcessBoundary],
        );
        expired.observed_at = Timestamp::from_millis(1);
        expired.valid_until = Timestamp::from_millis(2);
        provider.register_attestation(expired).unwrap();
        assert!(provider.supported_isolation_classes().is_empty());
    }

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
