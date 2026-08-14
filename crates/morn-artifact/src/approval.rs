//! Approvals of artifact versions.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ArtifactApprovalId, ArtifactVersionId, PrincipalId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ArtifactApprovalDecision {
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactApproval {
    pub id: ArtifactApprovalId,
    pub artifact_version_id: ArtifactVersionId,
    pub approver: PrincipalId,
    pub decision: ArtifactApprovalDecision,
    pub note: Option<String>,
    pub created_at: Timestamp,
}

impl ArtifactApproval {
    pub fn approve(
        artifact_version_id: ArtifactVersionId,
        approver: PrincipalId,
        note: Option<String>,
    ) -> Self {
        Self {
            id: ArtifactApprovalId::generate_with("artappr"),
            artifact_version_id,
            approver,
            decision: ArtifactApprovalDecision::Approved,
            note,
            created_at: Timestamp::now(),
        }
    }
}
