//! Object types and object instances in the Operational World.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ActionTypeId, ObjectId, ObjectTypeId, WorkspaceId};
use morn_kernel::status::LifecycleStatus;
use morn_kernel::time::Timestamp;

/// Domain definition of a class of operational objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectType {
    pub id: ObjectTypeId,
    pub name: String,
    pub workspace_id: WorkspaceId,
    pub properties: Vec<String>,
    pub allowed_states: Vec<String>,
    pub actions: Vec<ActionTypeId>,
    pub created_at: Timestamp,
}

impl ObjectType {
    pub fn new(
        name: impl Into<String>,
        workspace_id: WorkspaceId,
        properties: Vec<String>,
        allowed_states: Vec<String>,
        actions: Vec<ActionTypeId>,
    ) -> Self {
        Self {
            id: ObjectTypeId::generate_with("objt"),
            name: name.into(),
            workspace_id,
            properties,
            allowed_states,
            actions,
            created_at: Timestamp::now(),
        }
    }
}

/// An operational object instance.
///
/// `state` and `version` are private: canonical state may only change through
/// `WorldService::commit_state` (governed commits), never by direct mutation
/// from a runtime, harness or actor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectId,
    pub object_type_id: ObjectTypeId,
    pub workspace_id: WorkspaceId,
    state: BTreeMap<String, Value>,
    version: u64,
    pub status: LifecycleStatus,
    pub created_at: Timestamp,
}

impl Object {
    pub fn new(
        id: ObjectId,
        object_type_id: ObjectTypeId,
        workspace_id: WorkspaceId,
        initial_state: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            id,
            object_type_id,
            workspace_id,
            state: initial_state,
            version: 1,
            status: LifecycleStatus::Active,
            created_at: Timestamp::now(),
        }
    }

    /// Read-only access to the current state.
    pub fn state(&self) -> &BTreeMap<String, Value> {
        &self.state
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Apply a governed commit. Only callable within this crate (WorldService).
    pub(crate) fn apply_commit(&mut self, new_state: BTreeMap<String, Value>) -> u64 {
        self.state = new_state;
        self.version += 1;
        self.version
    }
}
