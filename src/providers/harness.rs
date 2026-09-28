//! harness — HarnessProvider: the unified minimal contract for agent harnesses.
//!
//! Every harness provider (DSH, Pi, fake, future) implements this one
//! contract; business layers depend only on it. Harness swap is proven by the
//! shared contract test (`tests/providers_g4g5.rs`).
//!
//! BOUNDARY: session events returned here are Harness evidence (master §8.1).
//! They must be mapped into Attempt/Receipt/Outcome by the control plane — a
//! harness event never directly becomes Work truth.

use crate::work::ExecutionBinding;
use serde::{Deserialize, Serialize};

/// Static capabilities a harness provider declares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessCapabilities {
    pub provider_id: String,
    pub version: String,
    /// Models / agent kinds the harness can run
    pub models: Vec<String>,
    /// True when the provider can actually reach its backend right now
    pub real_integration_available: bool,
    /// When false: what is missing (e.g. credentials, endpoint) — surfaced
    /// honestly instead of faked (Goal §8.2).
    pub integration_blocker: Option<String>,
}

/// A harness session started under a Binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessSession {
    pub session_ref: String,
    pub binding_ref: String,
    pub provider_id: String,
    pub created_at: String,
}

/// One input submission to a harness session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessInput {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_json: Option<String>,
}

/// The kind of a harness event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarnessEventKind {
    Started,
    Message,
    ToolCall,
    Candidate,
    Completed,
    Failed,
    Cancelled,
}

/// One harness session event (Harness evidence, not Work truth).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessEvent {
    pub seq: usize,
    pub kind: HarnessEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    pub recorded_at: String,
}

/// Usage accounting for a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessUsage {
    pub event_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_estimate: Option<i64>,
}

/// Harness health. `ExternalBlocked` is honest: no fake green.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarnessHealth {
    Healthy,
    Unavailable,
    ExternalBlocked,
}

/// HarnessProvider — the unified minimal contract (Goal §8.1).
///
/// The exact method set can be adjusted, but business layers only depend on
/// this contract.
pub trait HarnessProvider: Send + Sync {
    fn capabilities(&self) -> HarnessCapabilities;
    fn start_session(&self, binding: &ExecutionBinding) -> Result<HarnessSession, String>;
    fn resume_session(&self, session_ref: &str) -> Result<HarnessSession, String>;
    fn submit(&self, session: &HarnessSession, input: HarnessInput)
        -> Result<HarnessEvent, String>;
    fn cancel(&self, session: &HarnessSession) -> Result<(), String>;
    fn checkpoint(&self, session: &HarnessSession) -> Result<String, String>;
    fn get_events(
        &self,
        session: &HarnessSession,
        cursor: usize,
    ) -> Result<Vec<HarnessEvent>, String>;
    fn get_usage(&self, session: &HarnessSession) -> Result<HarnessUsage, String>;
    fn health(&self) -> HarnessHealth;
    fn close(&self) -> Result<(), String>;
}
