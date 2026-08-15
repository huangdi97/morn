//! Delegation and commitment.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{CommitmentId, DelegationId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// A formal delegation: delegator delegates scope, authority, budget and accountability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub id: DelegationId,
    pub workspace_id: WorkspaceId,
    pub delegator: String,
    pub delegatee: String,
    pub objective: String,
    pub task_scope: Vec<String>,
    pub authority_scope: Vec<String>,
    pub resource_scope: Vec<String>,
    pub time_budget: Option<String>,
    pub cost_budget: Option<String>,
    pub valid_from: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub can_redelegate: bool,
    pub retained_accountability: String,
}

impl Delegation {
    pub fn new(
        workspace_id: WorkspaceId,
        delegator: impl Into<String>,
        delegatee: impl Into<String>,
        objective: impl Into<String>,
    ) -> Self {
        Self {
            id: DelegationId::generate_with("del"),
            workspace_id,
            delegator: delegator.into(),
            delegatee: delegatee.into(),
            objective: objective.into(),
            task_scope: Vec::new(),
            authority_scope: Vec::new(),
            resource_scope: Vec::new(),
            time_budget: None,
            cost_budget: None,
            valid_from: Timestamp::now(),
            expires_at: None,
            can_redelegate: false,
            retained_accountability: "delegator retains final accountability".to_string(),
        }
    }

    pub fn with_authority_scope(mut self, scope: Vec<String>) -> Self {
        self.authority_scope = scope;
        self
    }

    pub fn with_task_scope(mut self, scope: Vec<String>) -> Self {
        self.task_scope = scope;
        self
    }

    pub fn with_expiry(mut self, expires_at: Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_redelegation(mut self, allowed: bool) -> Self {
        self.can_redelegate = allowed;
        self
    }

    /// Whether the delegation is currently valid (not expired).
    pub fn is_active(&self, now: Timestamp) -> bool {
        self.valid_from <= now && self.expires_at.map(|exp| now < exp).unwrap_or(true)
    }

    pub fn is_expired(&self, now: Timestamp) -> bool {
        !self.is_active(now)
    }

    /// Whether `action` is inside the delegated authority scope.
    pub fn authority_allows(&self, action: &str) -> bool {
        self.authority_scope.iter().any(|a| a == action)
    }

    /// Delegatee may only redelegate when explicitly allowed.
    pub fn may_redelegate(&self) -> Result<()> {
        if !self.can_redelegate {
            return Err(Error::not_authorized(format!(
                "delegation {} does not permit redelegation",
                self.id
            )));
        }
        Ok(())
    }
}

/// A commitment by a member to deliver an outcome for a work package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commitment {
    pub id: CommitmentId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub owner: String,
    pub expected_outputs: Vec<String>,
    pub deadline: Option<Timestamp>,
    pub status: String,
    pub blocked_reason: Option<String>,
}

impl Commitment {
    pub fn new(
        workspace_id: WorkspaceId,
        work_package_id: WorkPackageId,
        owner: impl Into<String>,
    ) -> Self {
        Self {
            id: CommitmentId::generate_with("com"),
            workspace_id,
            work_package_id,
            owner: owner.into(),
            expected_outputs: Vec::new(),
            deadline: None,
            status: "open".to_string(),
            blocked_reason: None,
        }
    }

    pub fn mark_blocked(&mut self, reason: impl Into<String>) {
        self.status = "blocked".to_string();
        self.blocked_reason = Some(reason.into());
    }

    pub fn complete(&mut self) {
        self.status = "complete".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_scope_and_expiry() {
        let ws = WorkspaceId::generate();
        let delegation = Delegation::new(ws, "pi", "analyst", "analysis")
            .with_authority_scope(vec![
                "start_analysis".to_string(),
                "submit_artifact".to_string(),
            ])
            .with_expiry(Timestamp::from_millis(Timestamp::now().millis() + 60_000));
        let now = Timestamp::now();
        let past = Timestamp::from_millis(now.millis() - 120_000);

        assert!(delegation.is_active(now));
        assert!(delegation.authority_allows("start_analysis"));
        assert!(!delegation.authority_allows("release_claim"));
        assert!(delegation.is_expired(past), "expired after expires_at");
        assert!(
            delegation.may_redelegate().is_err(),
            "redelegation not allowed by default"
        );
    }

    #[test]
    fn retained_accountability_is_not_transferred() {
        let ws = WorkspaceId::generate();
        let delegation = Delegation::new(ws, "pi", "analyst", "analysis");
        // Delegation != accountability transfer.
        assert!(delegation
            .retained_accountability
            .contains("delegator retains final accountability"));
    }
}
