//! Delegation and commitment.

use serde::{Deserialize, Serialize};

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
}
