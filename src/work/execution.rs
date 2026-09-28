//! execution — ExecutionBinding / ActionAttempt / Receipt / ReconciliationRecord.
//!
//! Binding rules (v11.5 master §10):
//! 1. An Attempt requires a Binding before it starts;
//! 2. An active Attempt pins its Binding;
//! 3. Provider hot-reload only affects future resolution, never the active
//!    Binding;
//! 4. Provider failure moves the Attempt to SUSPENDED/OUTCOME_UNKNOWN;
//! 5. Switching providers creates a new Binding (and a new Attempt when
//!    needed);
//! 6. An Attempt that already caused a side effect is reconciled first — it is
//!    never re-run from scratch under a new provider.
//!
//! Transport ack != business commit: a Receipt comes from an identifiable
//! source, and reconciliation prefers source-of-truth readback over any
//! self-description.

use serde::{Deserialize, Serialize};

/// Binding status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BindingStatus {
    #[default]
    Bound,
    Suspended,
    Revoked,
    Migrated,
}

/// ExecutionBinding — a first-class resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBinding {
    pub binding_id: String,
    pub work_ref: String,
    pub capability_release_digest: String,
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_environment_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_decision_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_ref: Option<String>,
    pub created_at: String,
    pub binding_generation: i64,
    #[serde(default)]
    pub status: BindingStatus,
}

impl ExecutionBinding {
    pub fn new(
        binding_id: &str,
        work_ref: &str,
        capability_release_digest: &str,
        provider_id: &str,
    ) -> Self {
        Self {
            binding_id: binding_id.to_string(),
            work_ref: work_ref.to_string(),
            capability_release_digest: capability_release_digest.to_string(),
            provider_id: provider_id.to_string(),
            provider_version: None,
            runtime_ref: None,
            execution_environment_ref: None,
            authority_decision_ref: None,
            profile_ref: None,
            site_ref: None,
            created_at: crate::work::now_iso(),
            binding_generation: 1,
            status: BindingStatus::Bound,
        }
    }
}

/// Attempt status machine. Failure/unknown branches are first-class: unknown
/// outcomes are reconciled, never auto-converted to failures, and committed
/// side effects are never re-executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttemptStatus {
    Proposed,
    Authorized,
    Dispatched,
    Acknowledged,
    Committed,
    Observed,
    Verified,
    FailedBeforeCommit,
    OutcomeUnknown,
    Reconciling,
    Compensating,
    Compensated,
    ManualInterventionRequired,
    Suspended,
}

/// ActionAttempt — one execution of an action under a Binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionAttempt {
    pub id: String,
    pub work_ref: String,
    pub binding_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_ref: Option<String>,
    pub action: String,
    pub idempotency_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_hash: Option<String>,
    pub status: AttemptStatus,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl ActionAttempt {
    pub fn proposed(
        id: &str,
        work_ref: &str,
        binding_ref: &str,
        action: &str,
        idempotency_key: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            work_ref: work_ref.to_string(),
            binding_ref: binding_ref.to_string(),
            capability_ref: None,
            action: action.to_string(),
            idempotency_key: idempotency_key.to_string(),
            business_key: None,
            source_system: None,
            request_hash: None,
            status: AttemptStatus::Proposed,
            created_at: crate::work::now_iso(),
            updated_at: None,
        }
    }
}

/// Receipt — evidence that an external action happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub attempt_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity_hash: Option<String>,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_summary: Option<serde_json::Value>,
}

impl Receipt {
    pub fn from_source(
        id: &str,
        attempt_ref: &str,
        source: &str,
        external_ref: Option<&str>,
    ) -> Self {
        Self {
            id: id.to_string(),
            attempt_ref: attempt_ref.to_string(),
            external_ref: external_ref.map(|s| s.to_string()),
            source: source.to_string(),
            integrity_hash: None,
            recorded_at: crate::work::now_iso(),
            provenance: None,
            payload_summary: None,
        }
    }
}

/// Reconciliation readback result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationFound {
    Yes,
    No,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationResult {
    Committed,
    NotCommitted,
    Unknown,
}

/// ReconciliationRecord — source-of-truth readback of an unknown outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationRecord {
    pub id: String,
    pub attempt_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<ReconciliationFound>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref_found: Option<String>,
    pub result: ReconciliationResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ReconciliationRecord {
    pub fn new(id: &str, attempt_ref: &str, result: ReconciliationResult) -> Self {
        Self {
            id: id.to_string(),
            attempt_ref: attempt_ref.to_string(),
            query: None,
            found: None,
            external_ref_found: None,
            result,
            decided_by: None,
            recorded_at: crate::work::now_iso(),
            notes: None,
        }
    }
}
