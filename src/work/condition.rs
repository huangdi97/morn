//! condition — Conditions are semantic gates, not fixed modules.
//!
//! Profile requirements declare which Conditions must hold; controllers and
//! providers produce them. A gate must never be silently bypassed: the
//! implementations that evaluate it are replaceable, the gate semantics are
//! not.

use serde::{Deserialize, Serialize};

/// The nine semantic gates defined by the v11.5 master.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConditionType {
    CapabilityResolved,
    AuthoritySatisfied,
    EvidenceSufficient,
    ExecutionBound,
    ActionCommitted,
    ExternalActionVerified,
    OutcomeObserved,
    AcceptanceSatisfied,
    ProfileConformant,
}

impl ConditionType {
    /// All gate types, for exhaustiveness checks and conformance suites.
    pub const ALL: [ConditionType; 9] = [
        ConditionType::CapabilityResolved,
        ConditionType::AuthoritySatisfied,
        ConditionType::EvidenceSufficient,
        ConditionType::ExecutionBound,
        ConditionType::ActionCommitted,
        ConditionType::ExternalActionVerified,
        ConditionType::OutcomeObserved,
        ConditionType::AcceptanceSatisfied,
        ConditionType::ProfileConformant,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionType::CapabilityResolved => "CapabilityResolved",
            ConditionType::AuthoritySatisfied => "AuthoritySatisfied",
            ConditionType::EvidenceSufficient => "EvidenceSufficient",
            ConditionType::ExecutionBound => "ExecutionBound",
            ConditionType::ActionCommitted => "ActionCommitted",
            ConditionType::ExternalActionVerified => "ExternalActionVerified",
            ConditionType::OutcomeObserved => "OutcomeObserved",
            ConditionType::AcceptanceSatisfied => "AcceptanceSatisfied",
            ConditionType::ProfileConformant => "ProfileConformant",
        }
    }
}

/// Condition status. `Unknown` is a first-class value: an unobserved gate is
/// not a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

/// One semantic gate observation on a resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "type")]
    pub type_: ConditionType,
    pub status: ConditionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<serde_json::Value>,
}

impl Condition {
    pub fn satisfied(condition_type: ConditionType) -> Self {
        Self {
            type_: condition_type,
            status: ConditionStatus::True,
            reason: None,
            observed_at: Some(crate::work::now_iso()),
            source_ref: None,
        }
    }

    pub fn unsatisfied(condition_type: ConditionType, reason: &str) -> Self {
        Self {
            type_: condition_type,
            status: ConditionStatus::False,
            reason: Some(reason.to_string()),
            observed_at: Some(crate::work::now_iso()),
            source_ref: None,
        }
    }

    pub fn unknown(condition_type: ConditionType, reason: &str) -> Self {
        Self {
            type_: condition_type,
            status: ConditionStatus::Unknown,
            reason: Some(reason.to_string()),
            observed_at: Some(crate::work::now_iso()),
            source_ref: None,
        }
    }

    pub fn is_true(&self) -> bool {
        self.status == ConditionStatus::True
    }
}
