//! Execution receipts: auditable proof that an external execution happened.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ExecutionReceiptId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub id: ExecutionReceiptId,
    pub workspace_id: WorkspaceId,
    pub session_id: String,
    pub trace_refs: Vec<String>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub outcome: String,
    pub harness_version: Option<Version>,
    pub runtime_version: Option<String>,
    pub event_ids: Vec<String>,
}

impl ExecutionReceipt {
    pub fn new(workspace_id: WorkspaceId, session_id: impl Into<String>) -> Self {
        Self {
            id: ExecutionReceiptId::generate_with("rcpt"),
            workspace_id,
            session_id: session_id.into(),
            trace_refs: Vec::new(),
            started_at: Timestamp::now(),
            ended_at: None,
            outcome: "running".to_string(),
            harness_version: None,
            runtime_version: None,
            event_ids: Vec::new(),
        }
    }
}
