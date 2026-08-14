//! Policy: rules governing who may do what, evaluated before action authorization.

use serde::{Deserialize, Serialize};

use crate::ids::{PolicyId, WorkspaceId};
use crate::status::LifecycleStatus;
use crate::time::Timestamp;
use crate::version::Version;

/// Whether a policy rule allows or denies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// A single policy rule. Empty subject/action/resource fields act as wildcards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRule {
    pub effect: PolicyEffect,
    pub subject: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub reason: Option<String>,
}

impl PolicyRule {
    pub fn allow(action: &str) -> Self {
        Self {
            effect: PolicyEffect::Allow,
            subject: None,
            action: Some(action.to_string()),
            resource: None,
            reason: None,
        }
    }

    pub fn deny(action: &str, reason: &str) -> Self {
        Self {
            effect: PolicyEffect::Deny,
            subject: None,
            action: Some(action.to_string()),
            resource: None,
            reason: Some(reason.to_string()),
        }
    }
}

/// Result of evaluating a policy set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
}

/// A versioned, immutable-by-convention policy document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub version: Version,
    pub rules: Vec<PolicyRule>,
    pub status: LifecycleStatus,
    pub created_at: Timestamp,
}

impl Policy {
    pub fn new(workspace_id: WorkspaceId, name: impl Into<String>, rules: Vec<PolicyRule>) -> Self {
        Self {
            id: PolicyId::generate_with("pol"),
            workspace_id,
            name: name.into(),
            version: Version::v1(),
            rules,
            status: LifecycleStatus::Active,
            created_at: Timestamp::now(),
        }
    }

    /// First matching rule wins; deny beats allow by default (explicit deny).
    pub fn evaluate(&self, subject: &str, action: &str, resource: &str) -> PolicyDecision {
        for rule in &self.rules {
            if rule.action.as_deref().is_some_and(|a| a != action) {
                continue;
            }
            if rule
                .subject
                .as_deref()
                .is_some_and(|s| s != "*" && s != subject)
            {
                continue;
            }
            if rule
                .resource
                .as_deref()
                .is_some_and(|r| r != "*" && r != resource)
            {
                continue;
            }
            return match rule.effect {
                PolicyEffect::Allow => PolicyDecision::Allow,
                PolicyEffect::Deny => PolicyDecision::Deny(
                    rule.reason
                        .clone()
                        .unwrap_or_else(|| "denied by policy".to_string()),
                ),
            };
        }
        PolicyDecision::Deny("no matching allow rule".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::WorkspaceId;

    #[test]
    fn policy_allow_deny() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(
            ws.clone(),
            "biolab",
            vec![
                PolicyRule::deny("release_claim", "requires human approval"),
                PolicyRule::allow("start_analysis"),
            ],
        );
        assert!(matches!(
            policy.evaluate("analyst", "start_analysis", "dataset-1"),
            PolicyDecision::Allow
        ));
        assert!(matches!(
            policy.evaluate("analyst", "release_claim", "claim-1"),
            PolicyDecision::Deny(_)
        ));
        assert!(matches!(
            policy.evaluate("analyst", "unknown_action", "x"),
            PolicyDecision::Deny(_)
        ));
    }
}
