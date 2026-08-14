//! World events: facts that happened (not intentions).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{EventId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: EventId,
    pub event_type: String,
    pub workspace_id: WorkspaceId,
    pub related_objects: Vec<String>,
    pub principal: String,
    pub state_before: Option<BTreeMap<String, Value>>,
    pub state_after: Option<BTreeMap<String, Value>>,
    pub evidence_ref: Option<String>,
    pub created_at: Timestamp,
}
