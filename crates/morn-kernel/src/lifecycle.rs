//! Lifecycle tracking: who changed which object from which status to which status.

use serde::{Deserialize, Serialize};

use crate::ids::{LifecycleRecordId, WorkspaceId};
use crate::time::Timestamp;

/// A record of a lifecycle state transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleRecord {
    pub id: LifecycleRecordId,
    pub workspace_id: WorkspaceId,
    pub object_type: String,
    pub object_id: String,
    pub from_status: String,
    pub to_status: String,
    pub changed_by: String,
    pub reason: Option<String>,
    pub created_at: Timestamp,
}

/// Tracks lifecycle transitions for auditability.
#[derive(Debug, Clone, Default)]
pub struct LifecycleTracker {
    records: Vec<LifecycleRecord>,
}

impl LifecycleTracker {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        workspace_id: WorkspaceId,
        object_type: impl Into<String>,
        object_id: impl Into<String>,
        from_status: impl Into<String>,
        to_status: impl Into<String>,
        changed_by: impl Into<String>,
        reason: Option<String>,
    ) -> LifecycleRecord {
        let record = LifecycleRecord {
            id: LifecycleRecordId::generate_with("lcr"),
            workspace_id,
            object_type: object_type.into(),
            object_id: object_id.into(),
            from_status: from_status.into(),
            to_status: to_status.into(),
            changed_by: changed_by.into(),
            reason,
            created_at: Timestamp::now(),
        };
        self.records.push(record.clone());
        record
    }

    pub fn history(&self, object_type: &str, object_id: &str) -> Vec<&LifecycleRecord> {
        self.records
            .iter()
            .filter(|r| r.object_type == object_type && r.object_id == object_id)
            .collect()
    }
}
