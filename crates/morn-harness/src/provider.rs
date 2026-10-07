//! Harness providers: MornNativeHarness (reference) and DeepSeekHarnessProvider (boundary).

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};

use crate::context::RuntimeContext;
use crate::event::{ExecutionEvent, ExecutionEventKind};
use crate::receipt::ExecutionReceipt;
use crate::scope::CapabilityScope;
use morn_kernel::ids::ExecutionReceiptId;

/// A mounted provider handle; unmount performs E0 lifecycle cleanup only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHandle {
    pub provider: String,
    pub scope_id: String,
}

/// Snapshot of a harness session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HarnessSnapshot {
    pub session_id: String,
    pub status: String,
    pub current_step: String,
    pub last_event: String,
}

/// Output produced by a harness in response to an input.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HarnessOutput {
    pub session_id: String,
    pub text: String,
    pub proposal_ref: Option<String>,
}

/// A session handle returned by `start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessSession {
    pub id: String,
    pub provider: String,
    pub status: String,
}

/// The Morn harness provider seam. Runtime/harness can only produce events,
/// proposals and receipts — never mutate canonical Morn state directly.
pub trait HarnessProvider: Send + Sync {
    fn provider_name(&self) -> &str;

    fn mount(&mut self, scope: CapabilityScope) -> Result<ProviderHandle>;
    fn unmount(&mut self, handle: &ProviderHandle) -> Result<()>;

    fn start(&mut self, ctx: &RuntimeContext) -> Result<HarnessSession>;
    fn send(&mut self, session_id: &str, input: &str) -> Result<HarnessOutput>;
    fn stream_events(&self, session_id: &str) -> Vec<ExecutionEvent>;
    fn inspect(&self, session_id: &str) -> Result<HarnessSnapshot>;
    fn interrupt(&mut self, session_id: &str) -> Result<()>;
    fn resume(&mut self, session_id: &str) -> Result<()>;
    fn terminate(&mut self, session_id: &str) -> Result<ExecutionReceipt>;
}

// ---------------------------------------------------------------------------
// MornNativeHarness: deterministic reference harness used for tests and as the
// default local fallback.
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct MornNativeHarness {
    name: String,
    sessions: HashMap<String, SessionState>,
    scopes: Vec<CapabilityScope>,
}

#[derive(Debug)]
struct SessionState {
    ctx: RuntimeContext,
    status: String,
    step: u64,
    events: Vec<ExecutionEvent>,
    last_event: String,
}

impl MornNativeHarness {
    pub fn new() -> Self {
        Self {
            name: "morn-native".to_string(),
            sessions: HashMap::new(),
            scopes: Vec::new(),
        }
    }
}

impl Default for MornNativeHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl HarnessProvider for MornNativeHarness {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn mount(&mut self, scope: CapabilityScope) -> Result<ProviderHandle> {
        let handle = ProviderHandle {
            provider: self.name.clone(),
            scope_id: scope.id.to_string(),
        };
        self.scopes.push(scope);
        Ok(handle)
    }

    fn unmount(&mut self, handle: &ProviderHandle) -> Result<()> {
        // E0 lifecycle cleanup only: remove the registered scope. No assumption
        // that any real-world E2/E3 effect is reversed.
        let before = self.scopes.len();
        self.scopes.retain(|s| s.id.to_string() != handle.scope_id);
        if self.scopes.len() == before {
            return Err(Error::not_found(format!(
                "scope {} not mounted",
                handle.scope_id
            )));
        }
        Ok(())
    }

    fn start(&mut self, ctx: &RuntimeContext) -> Result<HarnessSession> {
        let session_id = format!("morn-native-{}", uuid::Uuid::new_v4());
        let event = ExecutionEvent::new(
            ctx.workspace_id.clone(),
            session_id.clone(),
            ExecutionEventKind::SessionStarted,
            "session started",
        );
        let state = SessionState {
            ctx: ctx.clone(),
            status: "running".to_string(),
            step: 0,
            events: vec![event],
            last_event: "session_started".to_string(),
        };
        self.sessions.insert(session_id.clone(), state);
        Ok(HarnessSession {
            id: session_id,
            provider: self.name.clone(),
            status: "running".to_string(),
        })
    }

    fn send(&mut self, session_id: &str, input: &str) -> Result<HarnessOutput> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        state.step += 1;
        let step = state.step;
        let ws = state.ctx.workspace_id.clone();
        state.events.push(ExecutionEvent::new(
            ws,
            session_id.to_string(),
            ExecutionEventKind::ModelResponse,
            format!("native harness produced response for step {step}"),
        ));
        state.last_event = format!("model_response:{step}");
        Ok(HarnessOutput {
            session_id: session_id.to_string(),
            text: format!("deterministic output for input {input:?} at step {step}"),
            proposal_ref: None,
        })
    }

    fn stream_events(&self, session_id: &str) -> Vec<ExecutionEvent> {
        self.sessions
            .get(session_id)
            .map(|s| s.events.clone())
            .unwrap_or_default()
    }

    fn inspect(&self, session_id: &str) -> Result<HarnessSnapshot> {
        let state = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        Ok(HarnessSnapshot {
            session_id: session_id.to_string(),
            status: state.status.clone(),
            current_step: state.step.to_string(),
            last_event: state.last_event.clone(),
        })
    }

    fn interrupt(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        state.status = "paused".to_string();
        let ws = state.ctx.workspace_id.clone();
        state.events.push(ExecutionEvent::new(
            ws,
            session_id.to_string(),
            ExecutionEventKind::Interrupted,
            "session interrupted",
        ));
        state.last_event = "interrupted".to_string();
        Ok(())
    }

    fn resume(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        if state.status != "paused" {
            return Err(Error::invalid_state(format!(
                "session {session_id} is not paused"
            )));
        }
        state.status = "running".to_string();
        let ws = state.ctx.workspace_id.clone();
        state.events.push(ExecutionEvent::new(
            ws,
            session_id.to_string(),
            ExecutionEventKind::Resumed,
            "session resumed",
        ));
        state.last_event = "resumed".to_string();
        Ok(())
    }

    fn terminate(&mut self, session_id: &str) -> Result<ExecutionReceipt> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        let ws = state.ctx.workspace_id.clone();
        state.status = "terminated".to_string();
        state.events.push(ExecutionEvent::new(
            ws.clone(),
            session_id.to_string(),
            ExecutionEventKind::Completed,
            "session completed",
        ));
        state.last_event = "completed".to_string();
        let event_ids: Vec<String> = state.events.iter().map(|e| e.id.to_string()).collect();
        Ok(ExecutionReceipt {
            id: ExecutionReceiptId::generate_with("rcpt"),
            workspace_id: ws,
            session_id: session_id.to_string(),
            trace_refs: event_ids.clone(),
            started_at: state
                .events
                .first()
                .map(|e| e.created_at)
                .unwrap_or_default(),
            ended_at: Some(morn_kernel::time::Timestamp::now()),
            outcome: "completed".to_string(),
            harness_version: None,
            runtime_version: Some("morn-native".to_string()),
            event_ids,
        })
    }
}

// ---------------------------------------------------------------------------
// DeepSeekHarnessProvider: provider boundary for the DeepSeek Harness.
//
// Honest status: DeepSeek now publishes the official DSH runtime and a public
// out-of-process SDK/JSON-RPC boundary. This Rust reference provider has not yet
// wired that external transport, so Fixture mode proves only the Morn-side
// contract. Real mode fails closed instead of pretending that a live DSH
// session exists.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DshMode {
    /// Deterministic fixture behavior for contract tests (Morn-side invariants).
    Fixture,
    /// Real provider operations; currently unavailable in this environment.
    Real,
}

#[derive(Debug)]
pub struct DeepSeekHarnessProvider {
    name: String,
    mode: DshMode,
    sessions: HashMap<String, SessionState>,
    scopes: Vec<CapabilityScope>,
}

impl DeepSeekHarnessProvider {
    pub fn new(mode: DshMode) -> Self {
        Self {
            name: "deepseek-harness".to_string(),
            mode,
            sessions: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn mode(&self) -> DshMode {
        self.mode
    }

    fn real_unavailable(&self) -> Error {
        Error::external(
            "DeepSeek Harness real SDK transport is not configured in this Morn build; \
             official DSH exists externally, but only the Morn-side fixture contract is wired",
        )
    }
}

impl HarnessProvider for DeepSeekHarnessProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn mount(&mut self, scope: CapabilityScope) -> Result<ProviderHandle> {
        let handle = ProviderHandle {
            provider: self.name.clone(),
            scope_id: scope.id.to_string(),
        };
        self.scopes.push(scope);
        Ok(handle)
    }

    fn unmount(&mut self, handle: &ProviderHandle) -> Result<()> {
        let before = self.scopes.len();
        self.scopes.retain(|s| s.id.to_string() != handle.scope_id);
        if self.scopes.len() == before {
            return Err(Error::not_found(format!(
                "scope {} not mounted",
                handle.scope_id
            )));
        }
        Ok(())
    }

    fn start(&mut self, ctx: &RuntimeContext) -> Result<HarnessSession> {
        match self.mode {
            DshMode::Fixture => {
                let session_id = format!("dsh-fixture-{}", uuid::Uuid::new_v4());
                let event = ExecutionEvent::new(
                    ctx.workspace_id.clone(),
                    session_id.clone(),
                    ExecutionEventKind::SessionStarted,
                    "dsh fixture session started",
                );
                let state = SessionState {
                    ctx: ctx.clone(),
                    status: "running".to_string(),
                    step: 0,
                    events: vec![event],
                    last_event: "session_started".to_string(),
                };
                self.sessions.insert(session_id.clone(), state);
                Ok(HarnessSession {
                    id: session_id,
                    provider: self.name.clone(),
                    status: "running".to_string(),
                })
            }
            DshMode::Real => Err(self.real_unavailable()),
        }
    }

    fn send(&mut self, session_id: &str, input: &str) -> Result<HarnessOutput> {
        match self.mode {
            DshMode::Fixture => {
                let state = self
                    .sessions
                    .get_mut(session_id)
                    .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                state.step += 1;
                let step = state.step;
                let ws = state.ctx.workspace_id.clone();
                state.events.push(ExecutionEvent::new(
                    ws,
                    session_id.to_string(),
                    ExecutionEventKind::ToolCompleted,
                    format!("dsh fixture tool completed for step {step}"),
                ));
                state.last_event = format!("tool_completed:{step}");
                Ok(HarnessOutput {
                    session_id: session_id.to_string(),
                    text: format!("dsh fixture output for {input:?}"),
                    proposal_ref: None,
                })
            }
            DshMode::Real => Err(self.real_unavailable()),
        }
    }

    fn stream_events(&self, session_id: &str) -> Vec<ExecutionEvent> {
        self.sessions
            .get(session_id)
            .map(|s| s.events.clone())
            .unwrap_or_default()
    }

    fn inspect(&self, session_id: &str) -> Result<HarnessSnapshot> {
        let state = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        Ok(HarnessSnapshot {
            session_id: session_id.to_string(),
            status: state.status.clone(),
            current_step: state.step.to_string(),
            last_event: state.last_event.clone(),
        })
    }

    fn interrupt(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        state.status = "paused".to_string();
        let ws = state.ctx.workspace_id.clone();
        state.events.push(ExecutionEvent::new(
            ws,
            session_id.to_string(),
            ExecutionEventKind::Interrupted,
            "dsh fixture session interrupted",
        ));
        state.last_event = "interrupted".to_string();
        Ok(())
    }

    fn resume(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        if state.status != "paused" {
            return Err(Error::invalid_state(format!(
                "session {session_id} is not paused"
            )));
        }
        state.status = "running".to_string();
        let ws = state.ctx.workspace_id.clone();
        state.events.push(ExecutionEvent::new(
            ws,
            session_id.to_string(),
            ExecutionEventKind::Resumed,
            "dsh fixture session resumed",
        ));
        state.last_event = "resumed".to_string();
        Ok(())
    }

    fn terminate(&mut self, session_id: &str) -> Result<ExecutionReceipt> {
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        let ws = state.ctx.workspace_id.clone();
        state.status = "terminated".to_string();
        state.events.push(ExecutionEvent::new(
            ws.clone(),
            session_id.to_string(),
            ExecutionEventKind::Completed,
            "dsh fixture session completed",
        ));
        state.last_event = "completed".to_string();
        let event_ids: Vec<String> = state.events.iter().map(|e| e.id.to_string()).collect();
        Ok(ExecutionReceipt {
            id: ExecutionReceiptId::generate_with("rcpt"),
            workspace_id: ws,
            session_id: session_id.to_string(),
            trace_refs: event_ids.clone(),
            started_at: state
                .events
                .first()
                .map(|e| e.created_at)
                .unwrap_or_default(),
            ended_at: Some(morn_kernel::time::Timestamp::now()),
            outcome: "completed".to_string(),
            harness_version: None,
            runtime_version: Some("deepseek-harness-fixture".to_string()),
            event_ids,
        })
    }
}
