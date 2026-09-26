//! authority — AuthorityRequest / AuthorityDecision / AuthorityEvidence.
//!
//! Authority is not tool permission. It expresses WHO acts on behalf of WHOM,
//! for WHICH WORK/OBJECT/SITE, within WHICH TIME WINDOW, with WHICH PARAMETERS
//! and BUDGET, under WHAT DELEGATION DEPTH. Policy decision and enforcement
//! are separated: the decision provider is replaceable (OPA / Cedar / IAM /
//! human approval), enforcement happens at the Action Gateway.

use serde::{Deserialize, Serialize};

/// Time window within which a decision is valid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,
}

/// Delegation depth: how far authority may be delegated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationDepth {
    pub current: i64,
    pub max: i64,
}

impl DelegationDepth {
    pub fn within_limit(&self) -> bool {
        self.current <= self.max
    }
}

/// AuthorityRequest — a structured request for authority to perform an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorityRequest {
    pub id: String,
    pub principal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acting_for: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_envelope: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_window: Option<TimeWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<crate::work::CostAmount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_depth: Option<DelegationDepth>,
    pub created_at: String,
}

impl AuthorityRequest {
    pub fn new(id: &str, principal: &str, action: &str) -> Self {
        Self {
            id: id.to_string(),
            principal: principal.to_string(),
            acting_for: None,
            work_ref: None,
            site: None,
            resource: None,
            action: action.to_string(),
            parameter_envelope: None,
            time_window: None,
            budget: None,
            preconditions: Vec::new(),
            approval_refs: Vec::new(),
            delegation_depth: None,
            created_at: crate::work::now_iso(),
        }
    }

    pub fn is_within_time_window(&self, now: &str) -> bool {
        match &self.time_window {
            None => true,
            Some(window) => {
                let after_ok = window
                    .not_after
                    .as_ref()
                    .map(|limit| now <= limit.as_str())
                    .unwrap_or(true);
                let before_ok = window
                    .not_before
                    .as_ref()
                    .map(|limit| now >= limit.as_str())
                    .unwrap_or(true);
                after_ok && before_ok
            }
        }
    }
}

/// AuthorityDecisionType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorityDecisionType {
    Grant,
    Deny,
    GrantWithConditions,
}

/// AuthorityDecision — produced by a policy provider or human approver.
///
/// SAFETY: a decision is deny-by-default in spirit — absence of a decision
/// means no authority. Revocation is explicit; expired decisions grant nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorityDecision {
    pub id: String,
    pub request_ref: String,
    pub decision: AuthorityDecisionType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
    pub decided_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub revoked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_reason: Option<String>,
}

impl AuthorityDecision {
    pub fn grant(id: &str, request_ref: &str, decided_by: &str) -> Self {
        Self {
            id: id.to_string(),
            request_ref: request_ref.to_string(),
            decision: AuthorityDecisionType::Grant,
            conditions: Vec::new(),
            reason: None,
            decided_by: Some(decided_by.to_string()),
            decided_at: crate::work::now_iso(),
            evidence_refs: Vec::new(),
            expires_at: None,
            revoked: false,
            revocation_reason: None,
        }
    }

    pub fn deny(id: &str, request_ref: &str, decided_by: &str, reason: &str) -> Self {
        Self {
            decision: AuthorityDecisionType::Deny,
            reason: Some(reason.to_string()),
            ..Self::grant(id, request_ref, decided_by)
        }
    }

    pub fn revoke(&mut self, reason: &str) {
        self.revoked = true;
        self.revocation_reason = Some(reason.to_string());
    }

    /// A decision authorizes an action only when granted, not revoked, and not
    /// expired at the given time.
    pub fn authorizes(&self, now: &str) -> bool {
        let not_expired = self
            .expires_at
            .as_ref()
            .map(|limit| now <= limit.as_str())
            .unwrap_or(true);
        self.decision != AuthorityDecisionType::Deny && !self.revoked && not_expired
    }
}

/// AuthorityEvidence — evidence attached to a decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorityEvidence {
    pub source: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
}
