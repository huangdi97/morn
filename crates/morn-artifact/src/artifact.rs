//! Artifacts and their immutable versions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ArtifactId, ArtifactVersionId, PrincipalId, WorkspaceId};
use morn_kernel::status::ArtifactStatus;
use morn_kernel::time::Timestamp;

/// A logical artifact (a chain of immutable versions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub artifact_type: String,
    pub schema_version: String,
    pub workspace_id: WorkspaceId,
    pub created_by: PrincipalId,
    pub status: ArtifactStatus,
    pub current_version: u64,
    pub created_at: Timestamp,
}

/// One immutable artifact version. Content changes always create a new version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactVersion {
    pub id: ArtifactVersionId,
    pub artifact_id: ArtifactId,
    pub version_no: u64,
    pub content_ref: String,
    pub structured_content: Value,
    pub checksum: String,
    pub derived_from: Option<ArtifactVersionId>,
    pub supersedes: Option<ArtifactVersionId>,
    pub status: ArtifactStatus,
    pub created_by: PrincipalId,
    pub created_at: Timestamp,
}

impl Artifact {
    pub fn new(
        artifact_type: impl Into<String>,
        schema_version: impl Into<String>,
        workspace_id: WorkspaceId,
        created_by: PrincipalId,
    ) -> Self {
        Self {
            id: ArtifactId::generate_with("art"),
            artifact_type: artifact_type.into(),
            schema_version: schema_version.into(),
            workspace_id,
            created_by,
            status: ArtifactStatus::Draft,
            current_version: 0,
            created_at: Timestamp::now(),
        }
    }
}

impl ArtifactVersion {
    pub fn new(
        artifact_id: ArtifactId,
        version_no: u64,
        content_ref: impl Into<String>,
        structured_content: Value,
        checksum: impl Into<String>,
        derived_from: Option<ArtifactVersionId>,
        supersedes: Option<ArtifactVersionId>,
        created_by: PrincipalId,
    ) -> Self {
        Self {
            id: ArtifactVersionId::generate_with("artv"),
            artifact_id,
            version_no,
            content_ref: content_ref.into(),
            structured_content,
            checksum: checksum.into(),
            derived_from,
            supersedes,
            status: ArtifactStatus::Draft,
            created_by,
            created_at: Timestamp::now(),
        }
    }
}
