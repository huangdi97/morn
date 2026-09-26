//! capability — Capability supply chain semantics.
//!
//! Capability = Implementation + Manifest + Qualification + Authority Envelope
//! + Provenance. The four states (Declared / Observed / Qualified / Admitted)
//! must stay separated: a capability self-description is not a qualification,
//! and a qualification is not a site admission. Asset lifecycle and site
//! admission are separate stages.

use serde::{Deserialize, Serialize};

/// The four capability states, kept separated by contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityState {
    Declared,
    Observed,
    Qualified,
    Admitted,
    Deprecated,
    Revoked,
}

/// Asset lifecycle. Independent of site admission: SITE_BOUND is an admission
/// stage, not a build stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityLifecycle {
    Discovered,
    Extracted,
    Wrapped,
    Executable,
    Verified,
    Qualified,
    Released,
    SiteBound,
    Deprecated,
    Revoked,
}

/// What a capability does to the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SideEffects {
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Runtime / environment requirements a capability declares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionRequirements {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub harness_compatible: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_isolation: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network: Vec<String>,
}

/// Authority ceiling: maximum permissions a capability may ever request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorityCeiling {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maximum: Vec<String>,
}

/// Cost / latency envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Economics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_latency_p95_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_cost: Option<crate::work::CostAmount>,
}

/// CapabilityManifest — the declaration of a capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub name: String,
    pub version: String,
    pub digest: String,
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub spec: CapabilitySpec,
}

fn default_api_version() -> String {
    "capability.morn.io/v1alpha1".to_string()
}

/// Capability spec body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySpec {
    pub provides: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interfaces: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_effects: Option<SideEffects>,
    #[serde(default)]
    pub idempotency: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compensation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRequirements>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<AuthorityCeiling>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub economics: Option<Economics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
}

/// CapabilityRelease — a versioned, content-addressed release of a capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityRelease {
    pub name: String,
    pub version: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_ref: Option<String>,
    pub released_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    #[serde(default)]
    pub deprecated: bool,
}

/// QualificationRecord — evidence that a capability passed its qualification
/// suite in a specific environment. Qualification and acceptance must not be
/// modified by the evaluated subject itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualificationRecord {
    pub id: String,
    pub capability_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_evidence: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluator_identity: Option<String>,
    pub status: QualificationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualified_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<String>,
}

/// Qualification status. EXPIRED qualifications do not qualify new bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QualificationStatus {
    Passed,
    Failed,
    Expired,
    Pending,
}

/// SiteAdmission — a capability admitted to run at a specific site/profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteAdmission {
    pub id: String,
    pub capability_digest: String,
    pub site: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub admitted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_by: Option<String>,
    #[serde(default)]
    pub revoked: bool,
}
