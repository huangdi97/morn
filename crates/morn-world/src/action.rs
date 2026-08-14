//! Action types, proposals and executed actions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{
    ActionId, ActionProposalId, ActionTypeId, ObjectId, WorkspaceId,
};
use morn_kernel::time::Timestamp;

/// Status of an action through the governed lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ActionStatus {
    Proposed,
    Authorized,
    Executed,
    Failed,
    Recovered,
}

/// Domain definition of a governed state-changing action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionType {
    pub id: ActionTypeId,
    pub name: String,
    pub workspace_id: WorkspaceId,
    pub preconditions: Vec<String>,
    pub effects: Vec<String>,
    pub approval_required: Vec<String>,
    pub reversible: bool,
    pub verification_required: bool,
}

impl ActionType {
    pub fn new(
        name: impl Into<String>,
        workspace_id: WorkspaceId,
        preconditions: Vec<String>,
        effects: Vec<String>,
        approval_required: Vec<String>,
        reversible: bool,
        verification_required: bool,
    ) -> Self {
        Self {
            id: ActionTypeId::generate_with("actt"),
            name: name.into(),
            workspace_id,
            preconditions,
            effects,
            approval_required,
            reversible,
            verification_required,
        }
    }
}

/// A proposal to execute an action; the only thing actors/harnesses can create.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionProposal {
    pub id: ActionProposalId,
    pub action_type_id: ActionTypeId,
    pub target_object: ObjectId,
    pub workspace_id: WorkspaceId,
    pub params: BTreeMap<String, Value>,
    pub proposed_by: String,
    pub created_at: Timestamp,
}

impl ActionProposal {
    pub fn new(
        action_type_id: ActionTypeId,
        target_object: ObjectId,
        workspace_id: WorkspaceId,
        params: BTreeMap<String, Value>,
        proposed_by: impl Into<String>,
    ) -> Self {
        Self {
            id: ActionProposalId::generate_with("actp"),
            action_type_id,
            target_object,
            workspace_id,
            params,
            proposed_by: proposed_by.into(),
            created_at: Timestamp::now(),
        }
    }
}

/// An executed (or in-flight) action tied to its proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub id: ActionId,
    pub proposal_id: ActionProposalId,
    pub status: ActionStatus,
    pub executed_at: Option<Timestamp>,
    pub receipt_id: Option<String>,
}

impl Action {
    pub fn new(proposal_id: ActionProposalId) -> Self {
        Self {
            id: ActionId::generate_with("act"),
            proposal_id,
            status: ActionStatus::Proposed,
            executed_at: None,
            receipt_id: None,
        }
    }
}
