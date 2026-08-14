//! State diffs describing what changed on an object between committed states.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ActionId, ObjectId, StateSnapshotId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// One field-level change produced by a governed state commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDiff {
    pub id: StateSnapshotId,
    pub object_id: ObjectId,
    pub workspace_id: WorkspaceId,
    pub field: String,
    pub from: Option<Value>,
    pub to: Option<Value>,
    pub snapshot_id: StateSnapshotId,
    pub action_id: ActionId,
    pub created_at: Timestamp,
}
