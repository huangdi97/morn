//! State snapshots: immutable records of an object's state at a version.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ActionId, ObjectId, StateSnapshotId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: StateSnapshotId,
    pub object_id: ObjectId,
    pub workspace_id: WorkspaceId,
    pub state: BTreeMap<String, Value>,
    pub version: u64,
    pub created_by: String,
    pub created_at: Timestamp,
    pub reason: Option<String>,
    pub action_id: Option<ActionId>,
}
