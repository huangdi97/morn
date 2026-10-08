//! Harness providers: MornNativeHarness (reference) and DeepSeekHarnessProvider (boundary).

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};

use crate::context::RuntimeContext;
use crate::dsh_sdk::{DshSdkConfig, DshSdkStdioClient};
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

/// Provider feature negotiation. Optional lifecycle features are explicit:
/// Morn must never invent cancellation/resume semantics a wire protocol lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HarnessProviderFeatures {
    pub interrupt: bool,
    pub resume: bool,
    pub session_close: bool,
    pub durable_events: bool,
    pub multi_session: bool,
}

impl HarnessProviderFeatures {
    pub const fn full_reference() -> Self {
        Self {
            interrupt: true,
            resume: true,
            session_close: true,
            durable_events: true,
            multi_session: true,
        }
    }

    pub const fn dsh_sdk_current() -> Self {
        Self {
            interrupt: false,
            resume: false,
            session_close: false,
            durable_events: true,
            multi_session: true,
        }
    }

    pub const fn pi_rpc_current() -> Self {
        Self {
            interrupt: true,
            resume: false,
            session_close: false,
            durable_events: false,
            multi_session: false,
        }
    }
}

/// The Morn harness provider seam. Runtime/harness can only produce events,
/// proposals and receipts — never mutate canonical Morn state directly.
pub trait HarnessProvider: Send + Sync {
    fn provider_name(&self) -> &str;

    fn features(&self) -> HarnessProviderFeatures {
        HarnessProviderFeatures::full_reference()
    }

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
// DeepSeek publishes an official DSH runtime and SDK/JSON-RPC boundary.
// Real mode drives that exact stdio protocol when explicitly configured.
// Unsupported SDK lifecycle operations remain fail-closed, and all Harness
// output remains executor evidence rather than canonical Work truth.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DshMode {
    /// Deterministic fixture behavior for contract tests (Morn-side invariants).
    Fixture,
    /// Official DSH SDK subprocess transport; explicit isolated config required.
    Real,
}

#[derive(Debug)]
pub struct DeepSeekHarnessProvider {
    name: String,
    mode: DshMode,
    sessions: HashMap<String, SessionState>,
    scopes: Vec<CapabilityScope>,
    real_config: Option<DshSdkConfig>,
    real_client: Option<DshSdkStdioClient>,
}

impl DeepSeekHarnessProvider {
    pub fn new(mode: DshMode) -> Self {
        Self {
            name: "deepseek-harness".to_string(),
            mode,
            sessions: HashMap::new(),
            scopes: Vec::new(),
            real_config: None,
            real_client: None,
        }
    }

    pub fn with_real_sdk(config: DshSdkConfig) -> Self {
        let mut provider = Self::new(DshMode::Real);
        provider.real_config = Some(config);
        provider
    }

    pub fn from_real_env() -> Result<Self> {
        let config = DshSdkConfig::from_env()?;
        Ok(Self::with_real_sdk(config))
    }

    pub fn mode(&self) -> DshMode {
        self.mode
    }

    pub fn shutdown_real_runtime(&mut self) -> Result<()> {
        if self.mode != DshMode::Real {
            return Err(Error::invalid_state(
                "only a real DSH provider owns an SDK runtime",
            ));
        }
        if let Some(mut client) = self.real_client.take() {
            client.shutdown()?;
        }
        for state in self.sessions.values_mut() {
            if state.status != "terminated" {
                state.status = "runtime-closed".to_string();
                state.last_event = "runtime_closed".to_string();
            }
        }
        Ok(())
    }

    fn ensure_real_client(&mut self) -> Result<&mut DshSdkStdioClient> {
        if self.real_client.is_none() {
            let config = self
                .real_config
                .clone()
                .ok_or_else(|| self.real_unavailable())?;
            config.validate_for_real()?;
            let mut client = DshSdkStdioClient::spawn(&config)?;
            client.initialize(&config)?;
            self.real_client = Some(client);
        }
        self.real_client
            .as_mut()
            .ok_or_else(|| Error::internal("DSH SDK client missing after initialization"))
    }

    fn real_unavailable(&self) -> Error {
        Error::external(
            "DeepSeek Harness real SDK transport is not configured; use explicit DshSdkConfig or MORN_DSH_* deployment variables",
        )
    }
}

impl HarnessProvider for DeepSeekHarnessProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn features(&self) -> HarnessProviderFeatures {
        match self.mode {
            DshMode::Fixture => HarnessProviderFeatures::full_reference(),
            DshMode::Real => HarnessProviderFeatures::dsh_sdk_current(),
        }
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
            DshMode::Real => {
                self.ensure_real_client()?;
                let session_id = format!("morn-dsh-{}", uuid::Uuid::new_v4());
                let event = ExecutionEvent::new(
                    ctx.workspace_id.clone(),
                    session_id.clone(),
                    ExecutionEventKind::SessionStarted,
                    "DSH SDK runtime initialized; session reserved for lazy materialization",
                );
                self.sessions.insert(
                    session_id.clone(),
                    SessionState {
                        ctx: ctx.clone(),
                        status: "ready".to_string(),
                        step: 0,
                        events: vec![event],
                        last_event: "sdk_runtime_initialized".to_string(),
                    },
                );
                Ok(HarnessSession {
                    id: session_id,
                    provider: self.name.clone(),
                    status: "ready".to_string(),
                })
            }
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
            DshMode::Real => {
                if input.trim().is_empty() {
                    return Err(Error::validation("DSH prompt must be non-empty"));
                }
                {
                    let state = self
                        .sessions
                        .get_mut(session_id)
                        .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                    if !matches!(state.status.as_str(), "ready" | "idle") {
                        return Err(Error::invalid_state(format!(
                            "DSH session {session_id} cannot accept a new prompt from state {}",
                            state.status
                        )));
                    }
                    state.step += 1;
                    state.status = "running".to_string();
                    state.events.push(ExecutionEvent::new(
                        state.ctx.workspace_id.clone(),
                        session_id.to_string(),
                        ExecutionEventKind::ModelRequest,
                        format!("DSH SDK prompt admitted for step {}", state.step),
                    ));
                    state.last_event = format!("dsh_prompt:{}", state.step);
                }

                let run = self.ensure_real_client()?.run_text_prompt(session_id, input);
                let state = self
                    .sessions
                    .get_mut(session_id)
                    .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                match run {
                    Ok(run) => {
                        state.status = "idle".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::ModelResponse,
                            format!(
                                "DSH SDK turn completed; message={} finish={}",
                                run.message_id,
                                run.finish_reason.as_deref().unwrap_or("unknown")
                            ),
                        ));
                        state.last_event = format!(
                            "dsh_turn:{}",
                            run.finish_reason.as_deref().unwrap_or("unknown")
                        );
                        Ok(HarnessOutput {
                            session_id: session_id.to_string(),
                            text: run.final_response,
                            proposal_ref: None,
                        })
                    }
                    Err(error) => {
                        state.status = "outcome-unknown".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::Failed,
                            "DSH SDK turn lost definitive settlement; blind retry is forbidden",
                        ));
                        state.last_event = "dsh_turn_outcome_unknown".to_string();
                        Err(error)
                    }
                }
            }
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
        if self.mode == DshMode::Real {
            if !self.sessions.contains_key(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            return Err(Error::invalid_state(
                "current DSH SDK wire has no mid-turn cancel; close the owned runtime instead",
            ));
        }
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
        if self.mode == DshMode::Real {
            if !self.sessions.contains_key(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            return Err(Error::invalid_state(
                "current DSH SDK wire has no per-session resume; use ACP for persisted session resume",
            ));
        }
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
        if self.mode == DshMode::Real {
            if !self.sessions.contains_key(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            return Err(Error::invalid_state(
                "current DSH SDK wire has no per-session close; call shutdown_real_runtime for the owned process",
            ));
        }
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
