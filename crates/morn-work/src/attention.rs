//! Attention queue: where human attention is requested.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{AttentionItemId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AttentionKind {
    ApprovalRequired,
    PolicyConflict,
    EvidenceConflict,
    ToolFailure,
    BudgetRisk,
    DeadlineDrift,
    LowConfidence,
    IrreversibleAction,
    RepresentationBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AttentionPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionItem {
    pub id: AttentionItemId,
    pub workspace_id: WorkspaceId,
    pub kind: AttentionKind,
    pub subject: String,
    pub detail: String,
    pub priority: AttentionPriority,
    pub status: String,
    pub created_at: Timestamp,
    pub resolved_at: Option<Timestamp>,
}

impl AttentionItem {
    pub fn new(
        workspace_id: WorkspaceId,
        kind: AttentionKind,
        subject: impl Into<String>,
        detail: impl Into<String>,
        priority: AttentionPriority,
    ) -> Self {
        Self {
            id: AttentionItemId::generate_with("att"),
            workspace_id,
            kind,
            subject: subject.into(),
            detail: detail.into(),
            priority,
            status: "open".to_string(),
            created_at: Timestamp::now(),
            resolved_at: None,
        }
    }

    pub fn resolve(&mut self) {
        self.status = "resolved".to_string();
        self.resolved_at = Some(Timestamp::now());
    }
}
