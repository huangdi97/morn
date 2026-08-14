//! Decision packages: why a choice was made, based on what evidence, approved by whom.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ActionProposalId, ArtifactId, DecisionPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alternative {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRef {
    pub approver: String,
    pub decision: String,
    pub note: Option<String>,
}

/// A decision package ties evidence, alternatives, approval and outcomes together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionPackage {
    pub id: DecisionPackageId,
    pub workspace_id: WorkspaceId,
    pub intent: String,
    pub decision_subject: String,
    pub source_artifacts: Vec<ArtifactId>,
    pub alternatives: Vec<Alternative>,
    pub selected_option: Option<String>,
    pub reasoning_summary: String,
    pub assumptions: Vec<String>,
    pub approvals: Vec<ApprovalRef>,
    pub action_proposal: Option<ActionProposalId>,
    pub actual_state_diff: Option<String>,
    pub actual_outcome: Option<String>,
    pub created_at: Timestamp,
    pub context_snapshot: Value,
}

impl DecisionPackage {
    pub fn new(
        workspace_id: WorkspaceId,
        intent: impl Into<String>,
        decision_subject: impl Into<String>,
        source_artifacts: Vec<ArtifactId>,
    ) -> Self {
        Self {
            id: DecisionPackageId::generate_with("dec"),
            workspace_id,
            intent: intent.into(),
            decision_subject: decision_subject.into(),
            source_artifacts,
            alternatives: Vec::new(),
            selected_option: None,
            reasoning_summary: String::new(),
            assumptions: Vec::new(),
            approvals: Vec::new(),
            action_proposal: None,
            actual_state_diff: None,
            actual_outcome: None,
            created_at: Timestamp::now(),
            context_snapshot: serde_json::json!({}),
        }
    }
}
