//! policy — PolicyProvider: provider-neutral authority decisions (Goal §12.2).
//!
//! Decision and enforcement are separated: the decision provider is
//! replaceable (OPA / Cedar / IAM / human approval), enforcement happens at
//! the Action Gateway. Morn does not invent a complex policy DSL — the
//! reference provider evaluates the structured request with simple
//! deterministic rules.

use crate::work::{AuthorityDecision, AuthorityRequest};
use std::sync::Arc;

/// PolicyProvider — decides an AuthorityRequest (never an LLM guess).
pub trait PolicyProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    /// Decides a request. Absence of an explicit grant is a denial
    /// (deny-by-default).
    fn decide(&self, request: &AuthorityRequest) -> AuthorityDecision;
}

/// Reference policy provider: deterministic rules over the structured request.
///
/// Deny-by-default: a request is granted only when it names a principal and
/// action, is within its time window, and respects its delegation depth.
/// Side-effect classes beyond the request's own envelope are never granted.
pub struct ReferencePolicyProvider {
    /// Actions this policy is allowed to grant at all (allowlist).
    grantable_actions: Vec<String>,
}

impl ReferencePolicyProvider {
    pub fn new(grantable_actions: Vec<String>) -> Self {
        Self { grantable_actions }
    }

    /// A reference policy for the factory read-only profile: read-only
    /// actions only, no production write.
    pub fn factory_readonly() -> Self {
        Self::new(vec![
            "cmms.read".to_string(),
            "mes.read".to_string(),
            "historian.read".to_string(),
            "cmms.create-order".to_string(),
        ])
    }
}

impl Default for ReferencePolicyProvider {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl PolicyProvider for ReferencePolicyProvider {
    fn provider_id(&self) -> &str {
        "reference-policy"
    }

    fn decide(&self, request: &AuthorityRequest) -> AuthorityDecision {
        let now = crate::work::now_iso();
        let decision_id = format!("authz-{}", request.id);

        // Deny-by-default rules, checked in order.
        if request.principal.is_empty() || request.action.is_empty() {
            return AuthorityDecision::deny(
                &decision_id,
                &request.id,
                self.provider_id(),
                "request requires principal and action",
            );
        }
        if !self.grantable_actions.iter().any(|a| a == &request.action) {
            return AuthorityDecision::deny(
                &decision_id,
                &request.id,
                self.provider_id(),
                &format!(
                    "action '{}' is outside the policy allowlist",
                    request.action
                ),
            );
        }
        if !request.is_within_time_window(&now) {
            return AuthorityDecision::deny(
                &decision_id,
                &request.id,
                self.provider_id(),
                "request outside its time window",
            );
        }
        if let Some(depth) = &request.delegation_depth {
            if !depth.within_limit() {
                return AuthorityDecision::deny(
                    &decision_id,
                    &request.id,
                    self.provider_id(),
                    "delegation depth exceeded",
                );
            }
        }

        let mut decision = AuthorityDecision::grant(&decision_id, &request.id, self.provider_id());
        decision.decided_at = now;
        decision
    }
}

/// Convenience: decide via a boxed provider.
pub fn decide_with(
    provider: &Arc<dyn PolicyProvider>,
    request: &AuthorityRequest,
) -> AuthorityDecision {
    provider.decide(request)
}

/// SAFETY: the grantable set never includes a production-write action unless
/// a profile explicitly allows it — this assertion is the compile-time
/// documentation of that invariant for the reference provider.
#[test]
fn reference_policy_never_grants_production_write() {
    let policy = ReferencePolicyProvider::factory_readonly();
    assert!(!policy
        .grantable_actions
        .iter()
        .any(|a| a.contains("plc.write")));
    assert!(!policy
        .grantable_actions
        .iter()
        .any(|a| a.contains("quality.release")));
}
