//! Durable checkpoint: persisted work-run state for pause/resume/recovery.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{CheckpointId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// Serializable state of a work run at a point in time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub run_state: String,
    pub payload_json: String,
    pub created_at: Timestamp,
}

impl Checkpoint {
    pub fn new(
        workspace_id: WorkspaceId,
        work_package_id: WorkPackageId,
        run_state: impl Into<String>,
        payload_json: impl Into<String>,
    ) -> Self {
        Self {
            id: CheckpointId::generate_with("ckpt"),
            workspace_id,
            work_package_id,
            run_state: run_state.into(),
            payload_json: payload_json.into(),
            created_at: Timestamp::now(),
        }
    }
}
