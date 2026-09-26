//! work — Work resource: the unit of coordination.
//!
//! `Work.spec` is the desired state / contract; `Work.status` is the current
//! observation, not user expectation. Conditions are semantic gates: the
//! implementations that produce them are replaceable, the gates themselves are
//! not silently bypassable.

use crate::work::condition::Condition;
use serde::{Deserialize, Serialize};

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Money with an explicit unit (global rule: no unit-free financial values).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostAmount {
    pub amount: f64,
    pub currency: String,
}

impl CostAmount {
    pub fn new(amount: f64, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
        }
    }
}

/// Budget envelope for a Work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<CostAmount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_human_minutes: Option<i64>,
}

/// One acceptance requirement of a Work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceRequirement {
    pub criterion: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Acceptance spec: what must be true for the Work to be accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceSpec {
    pub required: Vec<AcceptanceRequirement>,
}

/// Authority requirements: which action classes need an AuthorityDecision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorityRequirement {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_for: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_ref: Option<String>,
}

/// Work.spec — desired state and running constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkSpec {
    pub work_type: String,
    pub goal: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_refs: Vec<crate::work::ObjectRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<AcceptanceSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<AuthorityRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<crate::work::SourceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default = "default_generation")]
    pub generation: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<String>,
}

fn default_generation() -> i64 {
    1
}

/// Work phase. `RECONCILING` is a first-class phase: unknown outcomes are not
/// auto-converted to failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkPhase {
    Draft,
    Resolving,
    AwaitingAuthority,
    Executing,
    Reconciling,
    Verifying,
    Completed,
    Cancelled,
    Blocked,
}

/// Acceptance state, driven by AcceptanceDecision records.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkAcceptanceState {
    #[default]
    Pending,
    Accepted,
    Rejected,
    Conditional,
}

/// Work.status — current observed state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkStatus {
    pub observed_generation: i64,
    pub phase: WorkPhase,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_binding_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_attempt_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_unknowns: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_outcome_refs: Vec<String>,
    #[serde(default)]
    pub acceptance_state: WorkAcceptanceState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocker_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Work — the protocol resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Work {
    pub id: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_generation")]
    pub generation: i64,
    pub spec: WorkSpec,
    pub status: WorkStatus,
}

fn default_kind() -> String {
    "Work".to_string()
}

fn default_api_version() -> String {
    "work.morn.io/v1alpha1".to_string()
}

impl Work {
    /// Creates a new Work in DRAFT phase with an empty status.
    pub fn draft(id: &str, spec: WorkSpec) -> Self {
        Self {
            id: id.to_string(),
            kind: default_kind(),
            api_version: default_api_version(),
            generation: spec.generation,
            spec,
            status: WorkStatus {
                observed_generation: 0,
                phase: WorkPhase::Draft,
                conditions: Vec::new(),
                active_binding_refs: Vec::new(),
                active_attempt_refs: Vec::new(),
                unresolved_unknowns: Vec::new(),
                observed_outcome_refs: Vec::new(),
                acceptance_state: WorkAcceptanceState::Pending,
                blocker_refs: Vec::new(),
                updated_at: Some(now_iso()),
            },
        }
    }

    /// Sets a condition (upsert by type) and bumps the timestamp.
    pub fn set_condition(&mut self, condition: Condition) {
        if let Some(existing) = self
            .status
            .conditions
            .iter_mut()
            .find(|c| c.type_ == condition.type_)
        {
            *existing = condition;
        } else {
            self.status.conditions.push(condition);
        }
        self.status.updated_at = Some(now_iso());
    }

    /// Returns the condition of a given type, if present.
    pub fn condition(&self, condition_type: crate::work::ConditionType) -> Option<&Condition> {
        self.status
            .conditions
            .iter()
            .find(|c| c.type_ == condition_type)
    }
}
