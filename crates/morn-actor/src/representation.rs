//! RepresentationContract: what a human-delegated actor may represent.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ActorInstanceId, PrincipalId, RepresentationContractId};
use morn_kernel::time::Timestamp;

/// Authority level of a representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RepresentationAuthority {
    Advisory,
    Drafting,
    Representative,
    Decisive,
}

/// A scoped representation contract for human-delegated actors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepresentationContract {
    pub id: RepresentationContractId,
    pub principal: PrincipalId,
    pub delegate_actor: ActorInstanceId,
    pub representation_scope: Vec<String>,
    pub cannot_represent: Vec<String>,
    pub authority_level: RepresentationAuthority,
    pub must_identify_as_ai: bool,
    pub valid_until: Option<Timestamp>,
    pub revoke_anytime: bool,
    pub accountable_principal: PrincipalId,
    pub status: String,
    pub revoked_at: Option<Timestamp>,
}

/// A scoped, validated subset of a representation contract for a given request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepresentationScope {
    pub contract_id: RepresentationContractId,
    pub action: String,
    pub allowed: bool,
    pub reason: String,
}

impl RepresentationContract {
    pub fn new(
        principal: PrincipalId,
        delegate_actor: ActorInstanceId,
        representation_scope: Vec<String>,
        cannot_represent: Vec<String>,
        authority_level: RepresentationAuthority,
        accountable_principal: PrincipalId,
    ) -> Self {
        Self {
            id: RepresentationContractId::generate_with("repr"),
            principal,
            delegate_actor,
            representation_scope,
            cannot_represent,
            authority_level,
            must_identify_as_ai: true,
            valid_until: None,
            revoke_anytime: true,
            accountable_principal,
            status: "active".to_string(),
            revoked_at: None,
        }
    }

    /// Revoke the contract (when revoke_anytime or after expiry).
    pub fn revoke(&mut self) -> morn_kernel::error::Result<()> {
        use morn_kernel::error::Error;
        if self.status == "revoked" {
            return Err(Error::invalid_state("contract already revoked"));
        }
        if !self.revoke_anytime {
            return Err(Error::not_authorized(
                "contract does not permit revocation at any time",
            ));
        }
        self.status = "revoked".to_string();
        self.revoked_at = Some(Timestamp::now());
        Ok(())
    }

    /// Check whether `action` falls inside the representation scope.
    pub fn check(&self, action: &str) -> RepresentationScope {
        if self.status == "revoked" {
            return RepresentationScope {
                contract_id: self.id.clone(),
                action: action.to_string(),
                allowed: false,
                reason: "representation contract is revoked".to_string(),
            };
        }
        if self.cannot_represent.iter().any(|a| a == action) {
            return RepresentationScope {
                contract_id: self.id.clone(),
                action: action.to_string(),
                allowed: false,
                reason: "action is explicitly excluded from representation".to_string(),
            };
        }
        if self.representation_scope.iter().any(|a| a == action) {
            return RepresentationScope {
                contract_id: self.id.clone(),
                action: action.to_string(),
                allowed: true,
                reason: "action is inside the representation scope".to_string(),
            };
        }
        RepresentationScope {
            contract_id: self.id.clone(),
            action: action.to_string(),
            allowed: false,
            reason: "action is outside the representation scope".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representation_allow_deny_revoke() {
        let principal = PrincipalId::generate();
        let actor = ActorInstanceId::generate();
        let mut contract = RepresentationContract::new(
            principal.clone(),
            actor,
            vec!["proposal_feedback".to_string()],
            vec!["board_vote".to_string(), "legal_signature".to_string()],
            RepresentationAuthority::Advisory,
            principal,
        );
        assert!(contract.check("proposal_feedback").allowed);
        assert!(!contract.check("board_vote").allowed);
        contract.revoke().unwrap();
        assert_eq!(contract.status, "revoked");
        assert!(contract.revoked_at.is_some());
        // After revoke nothing is allowed.
        assert!(!contract.check("proposal_feedback").allowed);
        assert!(contract.revoke().is_err(), "double revoke must fail");
    }

    #[test]
    fn representation_boundary_enforced() {
        let principal = PrincipalId::generate();
        let actor = ActorInstanceId::generate();
        let contract = RepresentationContract::new(
            principal.clone(),
            actor,
            vec!["proposal_feedback".to_string()],
            vec!["board_vote".to_string(), "legal_signature".to_string()],
            RepresentationAuthority::Advisory,
            principal,
        );
        assert!(contract.check("proposal_feedback").allowed);
        assert!(!contract.check("board_vote").allowed);
        assert!(!contract.check("hiring_decision").allowed);
    }
}
