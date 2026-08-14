//! Recovery records for interrupted work runs.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{RecoveryRecordId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryRecord {
    pub id: RecoveryRecordId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub failure: String,
    pub recovered: bool,
    pub resume_from: Option<String>,
    pub created_at: Timestamp,
}

impl RecoveryRecord {
    pub fn new(
        workspace_id: WorkspaceId,
        work_package_id: WorkPackageId,
        failure: impl Into<String>,
    ) -> Self {
        Self {
            id: RecoveryRecordId::generate_with("rec"),
            workspace_id,
            work_package_id,
            failure: failure.into(),
            recovered: false,
            resume_from: None,
            created_at: Timestamp::now(),
        }
    }
}
