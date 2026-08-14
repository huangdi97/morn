//! Normalized Morn execution events (harness/runtime facts, not canonical state).

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ExecutionEventId, SessionIdTag, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// Kinds of normalized execution events (per DSH_SPIKE event mapping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ExecutionEventKind {
    SessionStarted,
    ModelRequest,
    ModelResponse,
    ToolProposed,
    ToolStarted,
    ToolCompleted,
    ToolFailed,
    Checkpoint,
    Interrupted,
    Resumed,
    Completed,
    Failed,
}

/// A normalized execution event. Never stores private chain-of-thought;
/// stores auditable summaries, hashes/refs, versions and the receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub id: ExecutionEventId,
    pub workspace_id: WorkspaceId,
    pub session_id: String,
    pub kind: ExecutionEventKind,
    pub summary: String,
    pub input_hash: Option<String>,
    pub output_hash: Option<String>,
    pub refs: Vec<String>,
    pub harness_version: Option<Version>,
    pub model_version: Option<String>,
    pub created_at: Timestamp,
}

impl ExecutionEvent {
    pub fn new(
        workspace_id: WorkspaceId,
        session_id: impl Into<String>,
        kind: ExecutionEventKind,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: ExecutionEventId::generate_with("exev"),
            workspace_id,
            session_id: session_id.into(),
            kind,
            summary: summary.into(),
            input_hash: None,
            output_hash: None,
            refs: Vec::new(),
            harness_version: None,
            model_version: None,
            created_at: Timestamp::now(),
        }
    }
}

/// Re-export placeholder for session ids.
pub type SessionId = morn_kernel::ids::Id<SessionIdTag>;

