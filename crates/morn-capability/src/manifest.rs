//! Machine-readable capability manifest.
//!
//! A capability is broader than an agent: rules, programs, solvers, models,
//! humans, devices and hybrids can all be described and qualified uniformly.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{CapabilityId, Id};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::effect::EffectClass;
use crate::registry::CapabilityKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapabilityManifestTag;
pub type CapabilityManifestId = Id<CapabilityManifestTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum IsolationLevel {
    NoIsolation,
    Process,
    Container,
    MicroVm,
    FullVm,
    Remote,
    Physical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityInterface {
    pub protocol: String,
    pub input_schema_ref: String,
    pub output_schema_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRequirements {
    pub minimum_isolation: IsolationLevel,
    pub os: Option<String>,
    pub runtime_kinds: Vec<String>,
    pub harness_compatibility: Vec<String>,
    pub network_allowlist: Vec<String>,
    pub writable_paths: Vec<String>,
    pub secret_refs: Vec<String>,
    pub cpu_millis: Option<u64>,
    pub memory_mb: Option<u64>,
    pub gpu_count: Option<u32>,
    pub persistence_scope: String,
    pub timeout_ms: Option<u64>,
    pub side_effect_policy: String,
}

impl Default for ExecutionRequirements {
    fn default() -> Self {
        Self {
            minimum_isolation: IsolationLevel::NoIsolation,
            os: None,
            runtime_kinds: Vec::new(),
            harness_compatibility: Vec::new(),
            network_allowlist: Vec::new(),
            writable_paths: Vec::new(),
            secret_refs: Vec::new(),
            cpu_millis: None,
            memory_mb: None,
            gpu_count: None,
            persistence_scope: "attempt".to_string(),
            timeout_ms: None,
            side_effect_policy: "profile-governed".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityEnvelope {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub maximum_effect: EffectClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityEconomics {
    pub latency_p95_ms: Option<u64>,
    pub estimated_cost_micros: Option<u64>,
    pub throughput_per_minute: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityProvenance {
    pub source_ref: String,
    pub source_digest: Option<String>,
    pub build_provenance_ref: Option<String>,
    pub sbom_ref: Option<String>,
    pub signature_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CapabilityStage {
    Declared,
    Observed,
    Qualified,
    Admitted,
    Suspended,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub id: CapabilityManifestId,
    pub definition_id: CapabilityId,
    pub name: String,
    pub version: Version,
    pub digest: Option<String>,
    pub provider_ref: String,
    pub kind: CapabilityKind,
    pub provides: Vec<String>,
    pub requires: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub interfaces: Vec<CapabilityInterface>,
    pub idempotency_key_required: bool,
    pub compensation_ref: Option<String>,
    pub execution: ExecutionRequirements,
    pub authority: AuthorityEnvelope,
    pub economics: CapabilityEconomics,
    pub provenance: CapabilityProvenance,
    pub owner: Option<String>,
    pub license: Option<String>,
    pub declared_at: Timestamp,
}

impl CapabilityManifest {
    pub fn new(
        definition_id: CapabilityId,
        name: impl Into<String>,
        provider_ref: impl Into<String>,
        kind: CapabilityKind,
        maximum_effect: EffectClass,
    ) -> Self {
        Self {
            id: CapabilityManifestId::generate_with("cmanifest"),
            definition_id,
            name: name.into(),
            version: Version::v1(),
            digest: None,
            provider_ref: provider_ref.into(),
            kind,
            provides: Vec::new(),
            requires: Vec::new(),
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            interfaces: Vec::new(),
            idempotency_key_required: false,
            compensation_ref: None,
            execution: ExecutionRequirements::default(),
            authority: AuthorityEnvelope {
                allow: Vec::new(),
                deny: Vec::new(),
                maximum_effect,
            },
            economics: CapabilityEconomics::default(),
            provenance: CapabilityProvenance::default(),
            owner: None,
            license: None,
            declared_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityAdmissionRef {
    pub admission_ref: String,
    pub site_ref: String,
    pub profile_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRecord {
    pub manifest: CapabilityManifest,
    pub stage: CapabilityStage,
    pub qualification_refs: Vec<String>,
    /// Legacy/site summary retained for compatibility and display.
    pub admitted_sites: Vec<String>,
    /// Exact site + profile admissions used by v11.5 governed resolution.
    pub admission_refs: Vec<CapabilityAdmissionRef>,
}

impl CapabilityRecord {
    pub fn new(manifest: CapabilityManifest) -> Self {
        Self {
            manifest,
            stage: CapabilityStage::Declared,
            qualification_refs: Vec::new(),
            admitted_sites: Vec::new(),
            admission_refs: Vec::new(),
        }
    }
}
