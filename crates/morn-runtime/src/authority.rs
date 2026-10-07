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
    pub acting_for: Option<String>,
    pub action: String,
    pub resource: String,
    pub work_ref: Option<String>,
    pub site_ref: Option<String>,
    pub scope: Vec<String>,
    pub parameter_envelope: BTreeMap<String, String>,
    pub not_before: Option<Timestamp>,
    pub expires_at: Option<Timestamp>,
    pub budget_micros: Option<u64>,
    pub preconditions: Vec<String>,
    pub approval_refs: Vec<String>,
    pub delegation_depth: u32,
    pub revoked: bool,
    pub context: BTreeMap<String, String>,
}

impl AuthorityRequest {
    pub fn new(
        principal: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Self {
        Self {
            principal: principal.into(),
            acting_for: None,
            action: action.into(),
            resource: resource.into(),
            work_ref: None,
            site_ref: None,
            scope: Vec::new(),
            parameter_envelope: BTreeMap::new(),
            not_before: None,
            expires_at: None,
            budget_micros: None,
            preconditions: Vec::new(),
            approval_refs: Vec::new(),
            delegation_depth: 0,
            revoked: false,
            context: BTreeMap::new(),
        }
    }

    pub fn valid_now(&self, now: Timestamp) -> bool {
        if self.revoked {
            return false;
        }
        if self.not_before.is_some_and(|start| now < start) {
            return false;
        }
        if self.expires_at.is_some_and(|end| now > end) {
            return false;
        }
        true
    }
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
        if request.principal.trim().is_empty()
            || request.action.trim().is_empty()
            || request.resource.trim().is_empty()
        {
            return Err(morn_kernel::error::Error::validation(
                "authority request requires principal, action and resource",
            ));
        }
        if !request.valid_now(Timestamp::now()) {
            return Ok(AuthorityDecisionRecord {
                id: AuthorityDecisionId::generate_with("authz"),
                provider: self.provider_name().to_string(),
                allowed: false,
                reason: "authority request is revoked or outside its time window".to_string(),
                evidence_refs: vec![format!("policy:{}", self.policy.id)],
                decided_at: Timestamp::now(),
            });
        }
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
        let mut request = AuthorityRequest::new("investigator", "historian.read", "CNC-17");
        request.work_ref = Some("work-1042".to_string());
        request.site_ref = Some("plant-a".to_string());
        request.scope = vec!["historian.read".to_string()];
        let decision = provider.decide(&request).unwrap();
        assert!(decision.allowed);
        enforce_authority(&decision).unwrap();
    }

    #[test]
    fn revoked_authority_fails_closed_before_policy_allow() {
        let policy = Policy::new(
            WorkspaceId::generate(),
            "allow-read",
            vec![PolicyRule::allow("historian.read")],
        );
        let provider = NativePolicyAuthority::new(policy);
        let mut request = AuthorityRequest::new("agent", "historian.read", "CNC-17");
        request.revoked = true;
        let decision = provider.decide(&request).unwrap();
        assert!(!decision.allowed);
        assert!(decision.reason.contains("revoked"));
    }

    #[test]
    fn enforcement_fails_closed_on_denial() {
        let policy = Policy::new(WorkspaceId::generate(), "deny-by-default", vec![]);
        let provider = NativePolicyAuthority::new(policy);
        let request = AuthorityRequest::new("agent", "cmms.write", "plant-a");
        let decision = provider.decide(&request).unwrap();
        assert!(!decision.allowed);
        assert!(enforce_authority(&decision).is_err());
    }
}
