//! Reviews of artifact versions.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ArtifactVersionId, PrincipalId, ReviewId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ReviewDecision {
    Approve,
    RequestChanges,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub id: ReviewId,
    pub artifact_version_id: ArtifactVersionId,
    pub reviewer: PrincipalId,
    pub decision: ReviewDecision,
    pub comments: String,
    pub created_at: Timestamp,
}

impl Review {
    pub fn new(
        artifact_version_id: ArtifactVersionId,
        reviewer: PrincipalId,
        decision: ReviewDecision,
        comments: impl Into<String>,
    ) -> Self {
        Self {
            id: ReviewId::generate_with("rev"),
            artifact_version_id,
            reviewer,
            decision,
            comments: comments.into(),
            created_at: Timestamp::now(),
        }
    }
}
