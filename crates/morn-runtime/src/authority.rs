//! Provider-neutral authority decision seam.
//!
//! Policy decision and policy enforcement are deliberately separate. OPA,
//! Cedar or a customer IAM adapter can implement this contract later, while
//! the Action Gateway remains the enforcement point.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::Result;
use morn_kernel::ids::Id;
use morn_kernel::policy::{Policy, PolicyDecision};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AuthorityDecisionTag;
pub type AuthorityDecisionId = Id<AuthorityDecisionTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRequest {
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub work_ref: Option<String>,
    pub context: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityDecisionRecord {
    pub id: AuthorityDecisionId,
    pub provider: String,
    pub allowed: bool,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub decided_at: Timestamp,
}

pub trait AuthorityProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn decide(&self, request: &AuthorityRequest) -> Result<AuthorityDecisionRecord>;
}

#[derive(Debug, Clone)]
pub struct NativePolicyAuthority {
    pub policy: Policy,
}

impl NativePolicyAuthority {
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }
}

impl AuthorityProvider for NativePolicyAuthority {
    fn provider_name(&self) -> &str {
        "morn-native-policy"
    }

    fn decide(&self, request: &AuthorityRequest) -> Result<AuthorityDecisionRecord> {
        let decision = self
            .policy
            .evaluate(&request.principal, &request.action, &request.resource);
        let (allowed, reason) = match decision {
            PolicyDecision::Allow => (true, "allowed by policy".to_string()),
            PolicyDecision::Deny(reason) => (false, reason),
        };
        Ok(AuthorityDecisionRecord {
            id: AuthorityDecisionId::generate_with("authz"),
            provider: self.provider_name().to_string(),
            allowed,
            reason,
            evidence_refs: vec![format!("policy:{}", self.policy.id)],
            decided_at: Timestamp::now(),
        })
    }
}

pub fn enforce_authority(decision: &AuthorityDecisionRecord) -> Result<()> {
    if decision.allowed {
        Ok(())
    } else {
        Err(morn_kernel::error::Error::not_authorized(format!(
            "authority provider {} denied action: {}",
            decision.provider, decision.reason
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::WorkspaceId;
    use morn_kernel::policy::PolicyRule;

    #[test]
    fn native_policy_is_one_replaceable_authority_provider() {
        let policy = Policy::new(
            WorkspaceId::generate(),
            "factory",
            vec![PolicyRule::allow("historian.read")],
        );
        let provider = NativePolicyAuthority::new(policy);
        let request = AuthorityRequest {
            principal: "investigator".to_string(),
            action: "historian.read".to_string(),
            resource: "CNC-17".to_string(),
            work_ref: Some("work-1042".to_string()),
            context: BTreeMap::new(),
        };
        let decision = provider.decide(&request).unwrap();
        assert!(decision.allowed);
        enforce_authority(&decision).unwrap();
    }

    #[test]
    fn enforcement_fails_closed_on_denial() {
        let policy = Policy::new(WorkspaceId::generate(), "deny-by-default", vec![]);
        let provider = NativePolicyAuthority::new(policy);
        let request = AuthorityRequest {
            principal: "agent".to_string(),
            action: "cmms.write".to_string(),
            resource: "plant-a".to_string(),
            work_ref: None,
            context: BTreeMap::new(),
        };
        let decision = provider.decide(&request).unwrap();
        assert!(!decision.allowed);
        assert!(enforce_authority(&decision).is_err());
    }
}
