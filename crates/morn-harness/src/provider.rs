//! Harness providers: MornNativeHarness (reference) and DeepSeekHarnessProvider (boundary).

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};

use crate::context::RuntimeContext;
use crate::dsh_sdk::{DshNotification, DshSdkConfig, DshSdkStdioClient};
use crate::event::{ExecutionEvent, ExecutionEventKind};
use crate::receipt::ExecutionReceipt;
use crate::scope::CapabilityScope;
use morn_kernel::ids::{ExecutionReceiptId, WorkspaceId};
use morn_kernel::time::Timestamp;

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

pub const HARNESS_RUNTIME_HEALTH_LEASE_MS: i64 = 300_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessRuntimeHealthState {
    Fixture,
    Unconfigured,
    Configured,
    Initialized,
    Healthy,
    Degraded,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HarnessRuntimeHealth {
    pub state: HarnessRuntimeHealthState,
    pub settled_turns: u64,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub health_valid_until: Option<Timestamp>,
}

impl HarnessRuntimeHealth {
    fn new(state: HarnessRuntimeHealthState, reason: impl Into<String>) -> Self {
        Self {
            state,
            settled_turns: 0,
            reason: reason.into(),
            evidence_refs: Vec::new(),
            observed_at: Timestamp::now(),
            health_valid_until: None,
        }
    }

    pub fn fixture(reason: impl Into<String>) -> Self {
        Self::new(HarnessRuntimeHealthState::Fixture, reason)
    }

    pub fn unconfigured(reason: impl Into<String>) -> Self {
        Self::new(HarnessRuntimeHealthState::Unconfigured, reason)
    }

    pub fn configured(reason: impl Into<String>) -> Self {
        Self::new(HarnessRuntimeHealthState::Configured, reason)
    }

    pub fn mark_initialized(&mut self, reason: impl Into<String>, evidence_ref: impl Into<String>) {
        self.state = HarnessRuntimeHealthState::Initialized;
        self.reason = reason.into();
        self.evidence_refs = vec![evidence_ref.into()];
        self.observed_at = Timestamp::now();
        self.health_valid_until = Some(Timestamp::from_millis(
            self.observed_at
                .millis()
                .saturating_add(HARNESS_RUNTIME_HEALTH_LEASE_MS),
        ));
    }

    pub fn mark_live_turn(&mut self, reason: impl Into<String>, evidence_ref: impl Into<String>) {
        self.state = HarnessRuntimeHealthState::Healthy;
        self.settled_turns = self.settled_turns.saturating_add(1);
        self.reason = reason.into();
        self.evidence_refs = vec![evidence_ref.into()];
        self.observed_at = Timestamp::now();
        self.health_valid_until = Some(Timestamp::from_millis(
            self.observed_at
                .millis()
                .saturating_add(HARNESS_RUNTIME_HEALTH_LEASE_MS),
        ));
    }

    pub fn mark_degraded(&mut self, reason: impl Into<String>) {
        self.state = HarnessRuntimeHealthState::Degraded;
        self.reason = reason.into();
        // Evidence refs describe the *current* health observation. Carrying a
        // prior successful-turn reference across degradation would make status
        // APIs appear to cite stale success as evidence for the failure state.
        self.evidence_refs.clear();
        self.observed_at = Timestamp::now();
        self.health_valid_until = None;
    }

    pub fn mark_closed(&mut self, reason: impl Into<String>) {
        self.state = HarnessRuntimeHealthState::Closed;
        self.reason = reason.into();
        self.evidence_refs.clear();
        self.observed_at = Timestamp::now();
        self.health_valid_until = None;
    }

    pub fn selectable_at(&self, now: Timestamp) -> bool {
        self.state == HarnessRuntimeHealthState::Healthy
            && self.health_valid_until.is_some_and(|until| now <= until)
    }

    /// A freshly initialized real transport may perform its first governed
    /// turn. After that, only a live settled-turn health lease can authorize
    /// another turn. Both proofs are time-bounded.
    pub fn ready_for_new_turn_at(&self, now: Timestamp) -> bool {
        matches!(
            self.state,
            HarnessRuntimeHealthState::Initialized | HarnessRuntimeHealthState::Healthy
        ) && self.health_valid_until.is_some_and(|until| now <= until)
    }

    pub fn remaining_lease_ms(&self, now: Timestamp) -> Option<i64> {
        self.health_valid_until
            .map(|until| until.millis().saturating_sub(now.millis()))
            .filter(|remaining| *remaining > 0)
    }
}

/// The Morn harness provider seam. Runtime/harness can only produce events,
/// proposals and receipts — never mutate canonical Morn state directly.
pub trait HarnessProvider: Send + Sync {
    fn provider_name(&self) -> &str;

    fn features(&self) -> HarnessProviderFeatures {
        HarnessProviderFeatures::full_reference()
    }

    /// Scope restrictions required before this provider may start work.
    /// Generic contract/smoke runners consume this rather than guessing
    /// provider-specific admission rules.
    fn required_scope_restrictions(&self) -> &'static [&'static str] {
        &[]
    }

    /// Exact provider runtime version observed by the transport handshake when
    /// available. Absence must never be replaced with a guessed version.
    fn runtime_version(&self) -> Option<String> {
        None
    }

    /// Deployment-attested content identity of the exact provider distribution.
    /// Wire/server versions must never be substituted for this digest.
    fn runtime_digest(&self) -> Option<String> {
        None
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

    fn runtime_version(&self) -> Option<String> {
        Some("reference".to_string())
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
            work_package_id: Some(state.ctx.work_package_id.clone()),
            work_generation: state.ctx.work_generation,
            execution_binding_ref: state.ctx.execution_binding_ref.clone(),
            provider_ref: Some(self.name.clone()),
            scope_ref: state.ctx.scope_id.clone(),
            execution_environment_ref: state.ctx.execution_environment_ref.clone(),
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
            runtime_digest: None,
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

pub const DSH_REAL_E0_SCOPE_RESTRICTION: &str = "morn.effects<=E0";

#[derive(Debug)]
pub struct DeepSeekHarnessProvider {
    name: String,
    mode: DshMode,
    sessions: HashMap<String, SessionState>,
    scopes: Vec<CapabilityScope>,
    real_config: Option<DshSdkConfig>,
    real_client: Option<DshSdkStdioClient>,
    /// Deployment-attested DSH distribution identity; never sourced from SDK serverInfo.
    real_runtime_version: Option<String>,
    real_runtime_digest: Option<String>,
    /// Wire peer version observed from initialize.serverInfo. Protocol evidence only.
    real_wire_server_version: Option<String>,
    runtime_health: HarnessRuntimeHealth,
}

impl DeepSeekHarnessProvider {
    pub fn new(mode: DshMode) -> Self {
        let runtime_health = match mode {
            DshMode::Fixture => HarnessRuntimeHealth::fixture(
                "deterministic fixture contract; not a live external runtime",
            ),
            DshMode::Real => HarnessRuntimeHealth::unconfigured(
                "real DSH mode selected without an explicit SDK configuration",
            ),
        };
        Self {
            name: "deepseek-harness".to_string(),
            mode,
            sessions: HashMap::new(),
            scopes: Vec::new(),
            real_config: None,
            real_client: None,
            real_runtime_version: None,
            real_runtime_digest: None,
            real_wire_server_version: None,
            runtime_health,
        }
    }

    pub fn with_real_sdk(config: DshSdkConfig) -> Self {
        let mut provider = Self::new(DshMode::Real);
        provider.real_config = Some(config);
        provider.runtime_health = HarnessRuntimeHealth::configured(
            "real DSH SDK configuration accepted; live handshake not yet performed",
        );
        provider
    }

    pub fn from_real_env() -> Result<Self> {
        let config = DshSdkConfig::from_env()?;
        Ok(Self::with_real_sdk(config))
    }

    pub fn mode(&self) -> DshMode {
        self.mode
    }

    pub fn runtime_health(&self) -> &HarnessRuntimeHealth {
        &self.runtime_health
    }

    /// Deployment preflight for creating a real ExecutionBinding. This performs
    /// the official SDK initialize handshake but returns the deployment-attested
    /// DSH distribution version. initialize.serverInfo.version remains wire-only.
    pub fn preflight_real_runtime(&mut self) -> Result<String> {
        if self.mode != DshMode::Real {
            return Err(Error::invalid_state(
                "DSH runtime preflight is only valid in real mode",
            ));
        }
        let now = Timestamp::now();
        if self.real_client.is_some() && !self.runtime_health.ready_for_new_turn_at(now) {
            // The official SDK wire has no independent health/ping request.
            // Re-initialize through a new owned runtime rather than extending
            // stale evidence locally. Provider methods are mutex-serialized,
            // so this cannot race an in-flight send.
            self.shutdown_real_runtime()?;
        }
        self.ensure_real_client()?;
        self.real_runtime_version
            .clone()
            .ok_or_else(|| Error::external("DSH initialize did not expose a runtime version"))
    }

    pub fn configured_execution_environment_ref(&self) -> Option<&str> {
        self.real_config
            .as_ref()
            .and_then(|config| config.execution_environment_ref.as_deref())
    }

    pub fn configured_runtime_version(&self) -> Option<&str> {
        self.real_config
            .as_ref()
            .and_then(|config| config.runtime_version.as_deref())
    }

    pub fn configured_runtime_digest(&self) -> Option<&str> {
        self.real_runtime_digest.as_deref().or_else(|| {
            self.real_config
                .as_ref()
                .and_then(|config| config.runtime_digest.as_deref())
        })
    }

    pub fn configured_route_ref(&self) -> Option<String> {
        self.real_config
            .as_ref()
            .and_then(|config| config.route_ref().ok())
    }

    pub fn wire_server_version(&self) -> Option<&str> {
        self.real_wire_server_version.as_deref()
    }

    pub fn shutdown_real_runtime(&mut self) -> Result<()> {
        if self.mode != DshMode::Real {
            return Err(Error::invalid_state(
                "only a real DSH provider owns an SDK runtime",
            ));
        }
        // Taking the client guarantees Drop owns the final process reap even
        // when the protocol-level shutdown request fails. Projection cleanup
        // must therefore happen in this same call rather than requiring a
        // second shutdown attempt to make state truthful.
        let shutdown_error = self
            .real_client
            .take()
            .and_then(|mut client| client.shutdown().err());
        let close_reason = shutdown_error.as_ref().map_or_else(
            || "DSH SDK runtime was explicitly shut down".to_string(),
            |error| {
                format!("DSH SDK graceful shutdown failed; owned process was force-reaped: {error}")
            },
        );
        self.runtime_health.mark_closed(close_reason);
        self.real_runtime_version = None;
        self.real_runtime_digest = None;
        self.real_wire_server_version = None;
        for state in self.sessions.values_mut() {
            if state.status != "terminated" {
                state.status = "runtime-closed".to_string();
                state.last_event = "runtime_closed".to_string();
            }
        }
        match shutdown_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn ensure_real_client(&mut self) -> Result<&mut DshSdkStdioClient> {
        if self.real_client.is_none() {
            let setup = (|| {
                let config = self
                    .real_config
                    .clone()
                    .ok_or_else(|| self.real_unavailable())?;
                config.validate_for_real()?;
                let runtime_version = config.runtime_version.clone().ok_or_else(|| {
                    Error::validation("DSH runtime version missing after validation")
                })?;
                let runtime_digest = config.runtime_digest.clone().ok_or_else(|| {
                    Error::validation("DSH runtime digest missing after validation")
                })?;
                let mut client = DshSdkStdioClient::spawn(&config)?;
                let server = client.initialize(&config)?;
                Ok::<_, Error>((client, server, runtime_version, runtime_digest))
            })();
            match setup {
                Ok((client, server, runtime_version, runtime_digest)) => {
                    self.real_runtime_version = Some(runtime_version.clone());
                    self.real_runtime_digest = Some(runtime_digest.clone());
                    self.real_wire_server_version = Some(server.version.clone());
                    self.real_client = Some(client);
                    self.runtime_health.mark_initialized(
                        format!(
                            "official DSH SDK wire {} handshake succeeded; deployment pins distribution {} ({}) and no settled live turn yet",
                            server.version, runtime_version, runtime_digest
                        ),
                        format!("runtime://deepseek-harness/{runtime_version}#{runtime_digest}"),
                    );
                }
                Err(error) => {
                    self.runtime_health
                        .mark_degraded(format!("DSH SDK startup/initialize failed: {error}"));
                    return Err(error);
                }
            }
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

    fn require_real_e0_scope(&self, ctx: &RuntimeContext) -> Result<()> {
        let scope_id = ctx
            .scope_id
            .as_deref()
            .filter(|scope_id| !scope_id.trim().is_empty())
            .ok_or_else(|| {
                Error::validation("real DSH SDK requires an explicit RuntimeContext.scope_id")
            })?;
        let eligible = self.scopes.iter().any(|scope| {
            scope.workspace_id == ctx.workspace_id
                && scope.id.as_str() == scope_id
                && scope
                    .restrictions
                    .iter()
                    .any(|restriction| restriction == DSH_REAL_E0_SCOPE_RESTRICTION)
        });
        if eligible {
            Ok(())
        } else {
            Err(Error::validation(
                "real DSH SDK requires the exact mounted isolated E0 scope; E1/E2/E3 actions must use Morn ExternalAction",
            ))
        }
    }

    fn require_pinned_real_environment(&self, ctx: &RuntimeContext) -> Result<()> {
        if !ctx.proves_real_harness_environment() {
            return Err(Error::validation(
                "real DSH SDK requires a runtime-attested container-or-stronger environment with read/write policy, process boundary, resource limits, network-egress and secret indirection",
            ));
        }
        let configured = self
            .real_config
            .as_ref()
            .and_then(|config| config.execution_environment_ref.as_deref())
            .filter(|reference| !reference.trim().is_empty())
            .ok_or_else(|| {
                Error::validation("real DSH SDK configuration is missing execution_environment_ref")
            })?;
        if ctx.execution_environment_ref.as_deref() != Some(configured) {
            return Err(Error::validation(
                "real DSH SDK RuntimeContext execution environment does not match the pinned launch environment",
            ));
        }
        Ok(())
    }
}

fn dsh_notification_violates_e0(notification: &DshNotification) -> bool {
    if matches!(
        notification.method.as_str(),
        DSH_NOTIFICATION_SUBAGENT_STARTED | DSH_NOTIFICATION_SUBAGENT_FINISHED
    ) {
        return true;
    }
    notification.method == DSH_NOTIFICATION_SESSION_EVENT
        && notification
            .params
            .get("event")
            .and_then(|event| event.get("type"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|event_type| event_type.starts_with("tool/"))
}

fn normalize_dsh_notifications(
    workspace_id: &WorkspaceId,
    session_id: &str,
    notifications: &[DshNotification],
) -> Vec<ExecutionEvent> {
    let mut normalized = Vec::new();
    for notification in notifications {
        if notification.method != "session.event"
            || notification
                .params
                .get("sessionId")
                .and_then(serde_json::Value::as_str)
                != Some(session_id)
        {
            continue;
        }
        let Some(event) = notification.params.get("event") else {
            continue;
        };
        let Some(event_type) = event.get("type").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let data = event.get("data").unwrap_or(&serde_json::Value::Null);
        let (kind, summary) = match event_type {
            "request/header" => (
                ExecutionEventKind::ModelRequest,
                "DSH model request header committed".to_string(),
            ),
            "assistant/message" => (
                ExecutionEventKind::ModelResponse,
                "DSH assistant message committed".to_string(),
            ),
            "assistant/attempt" => (
                ExecutionEventKind::Failed,
                "DSH model attempt settled without a surface message".to_string(),
            ),
            "tool/call" => {
                let name = data
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                (
                    ExecutionEventKind::ToolProposed,
                    format!("DSH tool call proposed: {name}"),
                )
            }
            "tool/result" => {
                let failed = data.get("error").is_some()
                    || data
                        .get("message")
                        .and_then(|message| message.get("isError"))
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                (
                    if failed {
                        ExecutionEventKind::ToolFailed
                    } else {
                        ExecutionEventKind::ToolCompleted
                    },
                    if failed {
                        "DSH tool result committed as error".to_string()
                    } else {
                        "DSH tool result committed".to_string()
                    },
                )
            }
            "turn/end" => {
                let reason = data
                    .get("reason")
                    .and_then(|reason| reason.get("kind"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                (
                    ExecutionEventKind::Checkpoint,
                    format!("DSH durable turn/end: {reason}"),
                )
            }
            other if other.starts_with("tool/") => (
                ExecutionEventKind::ToolProposed,
                format!("DSH tool lifecycle event observed: {other}"),
            ),
            _ => continue,
        };
        let mut record =
            ExecutionEvent::new(workspace_id.clone(), session_id.to_string(), kind, summary);
        if event_type == "request/header" {
            record.model_version = data
                .get("header")
                .and_then(|header| header.get("config"))
                .and_then(|config| config.get("model"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
        }
        if let Some(call_id) = data
            .get("callId")
            .or_else(|| {
                data.get("message")
                    .and_then(|message| message.get("toolCallId"))
            })
            .and_then(serde_json::Value::as_str)
        {
            record.refs.push(format!("dsh-tool-call:{call_id}"));
        }
        normalized.push(record);
    }
    normalized
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

    fn required_scope_restrictions(&self) -> &'static [&'static str] {
        match self.mode {
            DshMode::Fixture => &[],
            DshMode::Real => &[DSH_REAL_E0_SCOPE_RESTRICTION],
        }
    }

    fn runtime_version(&self) -> Option<String> {
        match self.mode {
            DshMode::Fixture => Some("fixture".to_string()),
            DshMode::Real => self.real_runtime_version.clone(),
        }
    }

    fn runtime_digest(&self) -> Option<String> {
        match self.mode {
            DshMode::Fixture => None,
            DshMode::Real => self.real_runtime_digest.clone(),
        }
    }

    fn mount(&mut self, scope: CapabilityScope) -> Result<ProviderHandle> {
        if self.mode == DshMode::Real
            && !scope
                .restrictions
                .iter()
                .any(|restriction| restriction == DSH_REAL_E0_SCOPE_RESTRICTION)
        {
            return Err(Error::validation(
                "real DSH SDK scope must explicitly declare morn.effects<=E0",
            ));
        }
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
                if self.real_config.is_none() {
                    return Err(self.real_unavailable());
                }
                if !ctx.proves_pinned_work_execution() {
                    return Err(Error::validation(
                        "real DSH SDK execution requires exact Work generation and ExecutionBinding",
                    ));
                }
                self.require_real_e0_scope(ctx)?;
                self.require_pinned_real_environment(ctx)?;
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

                // Scope/environment admission is renewed for every prompt.
                // Unmounting/revoking the E0 execution scope must block future
                // turns before anything reaches the external harness.
                let ctx = self
                    .sessions
                    .get(session_id)
                    .ok_or_else(|| Error::not_found(format!("session {session_id}")))?
                    .ctx
                    .clone();
                self.require_real_e0_scope(&ctx)?;
                self.require_pinned_real_environment(&ctx)?;

                // Bootstrap/handshake happens before we mutate the session into
                // a running state. A configuration or initialize failure means
                // no prompt was dispatched and must not strand the session as
                // "running" or manufacture OUTCOME_UNKNOWN.
                self.ensure_real_client()?;

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

                let run = self
                    .real_client
                    .as_mut()
                    .ok_or_else(|| Error::internal("DSH SDK client missing after initialization"))?
                    .run_text_prompt(session_id, input);
                let state = self
                    .sessions
                    .get_mut(session_id)
                    .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                let mut prohibited_tool_activity = false;
                if let Ok(run) = &run {
                    let normalized = normalize_dsh_notifications(
                        &state.ctx.workspace_id,
                        session_id,
                        &run.notifications,
                    );
                    prohibited_tool_activity = run
                        .notifications
                        .iter()
                        .any(dsh_notification_violates_e0);

                    state.events.extend(normalized);
                }
                if prohibited_tool_activity {
                    // Real DSH is admitted only through Morn's E0 harness seam.
                    // Tool execution belongs behind governed Capability /
                    // ExternalAction boundaries. We cannot undo a tool that
                    // already ran inside the external runtime, so fail closed,
                    // reap the owned process and refuse to promote its output.
                    let _owned_runtime = self.real_client.take();
                    self.real_runtime_version = None;
                    self.real_runtime_digest = None;
                    self.real_wire_server_version = None;
                    self.runtime_health.mark_degraded(
                        "DSH SDK emitted tool activity on an E0-only provider path; owned runtime reaped",
                    );
                    state.status = "policy-violation-runtime-reaped".to_string();
                    state.events.push(ExecutionEvent::new(
                        state.ctx.workspace_id.clone(),
                        session_id.to_string(),
                        ExecutionEventKind::Failed,
                        "DSH E0 provider observed prohibited tool activity; output rejected and runtime reaped",
                    ));
                    state.last_event = "dsh_e0_tool_policy_violation".to_string();
                    return Err(Error::external(
                        "DSH E0 provider observed tool activity; executor output was rejected",
                    ));
                }
                match run {
                    Ok(run) if run.completed_successfully() => {
                        self.runtime_health.mark_live_turn(
                            "live DSH SDK turn completed successfully",
                            format!("runtime://deepseek-harness/session/{session_id}/settled-turn"),
                        );
                        state.status = "idle".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::Checkpoint,
                            format!(
                                "DSH SDK owned turn settled; message={} finish=completed",
                                run.message_id
                            ),
                        ));
                        state.last_event = "dsh_turn:completed".to_string();
                        Ok(HarnessOutput {
                            session_id: session_id.to_string(),
                            text: run.final_response,
                            proposal_ref: None,
                        })
                    }
                    Ok(run) => {
                        let reason = run.finish_reason.as_deref().unwrap_or("missing");
                        self.runtime_health.mark_degraded(format!(
                            "DSH SDK turn settled with non-success finish reason {reason}"
                        ));
                        state.status = "idle-non-success".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::Failed,
                            format!(
                                "DSH SDK turn settled without success; message={} finish={reason}",
                                run.message_id
                            ),
                        ));
                        state.last_event = format!("dsh_turn_non_success:{reason}");
                        Err(Error::external(format!(
                            "DSH turn settled with non-success finish reason {reason:?}; executor output was not promoted"
                        )))
                    }
                    Err(error) => {
                        // The current official SDK wire has no mid-turn cancel. Once
                        // settlement is lost, keeping the subprocess alive would permit
                        // unobserved background execution after Morn already returned an
                        // error. This provider is E0-only, so containment is to reap the
                        // owned runtime process and require a fresh handshake for any
                        // later Work/session.
                        let _owned_runtime = self.real_client.take();
                        self.real_runtime_version = None;
                        self.real_runtime_digest = None;
                        self.real_wire_server_version = None;
                        self.runtime_health.mark_degraded(format!(
                            "DSH SDK turn failed to settle; owned runtime reaped: {error}"
                        ));
                        state.status = "outcome-unknown-runtime-reaped".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::Failed,
                            "DSH SDK turn lost definitive settlement; owned runtime reaped and blind retry forbidden",
                        ));
                        state.last_event = "dsh_turn_outcome_unknown_runtime_reaped".to_string();
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
            work_package_id: Some(state.ctx.work_package_id.clone()),
            work_generation: state.ctx.work_generation,
            execution_binding_ref: state.ctx.execution_binding_ref.clone(),
            provider_ref: Some(self.name.clone()),
            scope_ref: state.ctx.scope_id.clone(),
            execution_environment_ref: state.ctx.execution_environment_ref.clone(),
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
            runtime_digest: None,
            event_ids,
        })
    }
}

#[cfg(test)]
mod dsh_provider_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn real_dsh_rejects_context_from_a_different_execution_environment() {
        use morn_kernel::ids::{ActorInstanceId, WorkPackageId};
        use morn_kernel::{ExecutionClass, ExecutionGuarantee};

        let root = std::env::temp_dir().join("morn-provider-env-pin");
        let config = DshSdkConfig::profile_sdk(
            root.join("workspace").to_string_lossy(),
            "deepseek-official",
            "deepseek-v4-flash",
        )
        .with_dsh_home(root.join("dsh-home").to_string_lossy())
        .with_execution_environment_ref("env://container/pinned")
        .with_runtime_identity(
            "fixture-runtime-1",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );
        let provider = DeepSeekHarnessProvider::with_real_sdk(config);
        let ctx = RuntimeContext::new(
            WorkspaceId::generate(),
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        )
        .with_execution_environment(
            "env://container/forged",
            ExecutionClass::Container,
            vec![
                ExecutionGuarantee::FilesystemReadPolicy,
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::ProcessBoundary,
                ExecutionGuarantee::ResourceLimits,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
                ExecutionGuarantee::RuntimeAttestation,
                ExecutionGuarantee::ToolMediation,
            ],
        )
        .unwrap();
        assert!(provider.require_pinned_real_environment(&ctx).is_err());

        let pinned = RuntimeContext {
            execution_environment_ref: Some("env://container/pinned".to_string()),
            ..ctx
        };
        assert!(provider.require_pinned_real_environment(&pinned).is_ok());
    }

    fn real_wire_fixture_provider() -> (DeepSeekHarnessProvider, RuntimeContext, ProviderHandle) {
        use morn_kernel::ids::{ActorInstanceId, WorkPackageId};
        use morn_kernel::{ExecutionClass, ExecutionGuarantee};

        let executable = std::env::current_exe().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let home =
            std::env::temp_dir().join(format!("morn-dsh-provider-home-{}", uuid::Uuid::new_v4()));
        let environment_ref = "env://container/dsh-provider-test";
        let mut config =
            DshSdkConfig::profile_sdk(cwd.to_string_lossy(), "fixture-provider", "fixture-model")
                .with_dsh_home(home.to_string_lossy())
                .with_execution_environment_ref(environment_ref)
                .with_runtime_identity(
                    "fixture-runtime-1",
                    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                );
        config.command = executable.to_string_lossy().to_string();
        config.args = vec![
            "--exact".to_string(),
            "dsh_sdk::tests::fake_sdk_runtime".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
        ];
        config.allow_protocol_fixture_transport();
        config.request_timeout_ms = 10_000;
        config.turn_timeout_ms = 10_000;

        let workspace = WorkspaceId::generate();
        let mut provider = DeepSeekHarnessProvider::with_real_sdk(config);
        let scope = CapabilityScope::new(
            crate::scope::ScopeKind::ExecutionRun,
            None,
            workspace.clone(),
            "real-dsh-wire-fixture",
        )
        .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
        let handle = provider.mount(scope).unwrap();
        let ctx = RuntimeContext::new(
            workspace,
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        )
        .with_work_binding(
            1,
            morn_kernel::ids::RuntimeBindingId::generate_with("binding"),
        )
        .unwrap()
        .with_scope_id(handle.scope_id.clone())
        .unwrap()
        .with_execution_environment(
            environment_ref,
            ExecutionClass::Container,
            vec![
                ExecutionGuarantee::FilesystemReadPolicy,
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::ProcessBoundary,
                ExecutionGuarantee::ResourceLimits,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
                ExecutionGuarantee::RuntimeAttestation,
                ExecutionGuarantee::ToolMediation,
            ],
        )
        .unwrap();
        (provider, ctx, handle)
    }

    #[test]
    fn real_dsh_wire_fixture_passes_capability_negotiated_contract_and_smoke() {
        let (mut provider, mut ctx, original_handle) = real_wire_fixture_provider();
        ctx.provenance_refs
            .push("evidence://real-dsh-wire-fixture".to_string());

        let contract = crate::contract::run_provider_contract(&mut provider, &ctx).unwrap();
        assert!(contract.all_passed(), "checks: {:?}", contract.checks);

        let smoke = crate::smoke::run_harness_smoke(&mut provider, &ctx).unwrap();
        assert!(smoke.all_ok(), "smoke: {smoke:?}");
        assert!(smoke.detail.contains("session close is unsupported"));

        provider.unmount(&original_handle).unwrap();
        provider.shutdown_real_runtime().unwrap();
    }

    #[test]
    fn real_dsh_e0_provider_rejects_tool_activity_and_reaps_runtime() {
        let (mut provider, ctx, _handle) = real_wire_fixture_provider();
        let session = provider.start(&ctx).unwrap();
        let error = provider.send(&session.id, "__tool_activity__").unwrap_err();
        assert!(format!("{error}").contains("tool activity"));
        assert!(provider.real_client.is_none());
        assert!(provider.runtime_version().is_none());
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Degraded
        );
        let snapshot = provider.inspect(&session.id).unwrap();
        assert_eq!(snapshot.status, "policy-violation-runtime-reaped");
        assert_eq!(snapshot.last_event, "dsh_e0_tool_policy_violation");
        let events = provider.stream_events(&session.id);
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ExecutionEventKind::ToolProposed | ExecutionEventKind::ToolCompleted
            )
        }));
        assert!(provider.send(&session.id, "blind retry forbidden").is_err());
    }

    #[test]
    fn real_dsh_preflight_refreshes_expired_initialization_by_restarting_owned_runtime() {
        let (mut provider, _ctx, _handle) = real_wire_fixture_provider();
        assert_eq!(
            provider.preflight_real_runtime().unwrap(),
            "fixture-runtime-1"
        );
        let first_observed = provider.runtime_health().observed_at;
        provider.runtime_health.health_valid_until = Some(Timestamp::from_millis(
            Timestamp::now().millis().saturating_sub(1),
        ));
        assert!(!provider
            .runtime_health()
            .ready_for_new_turn_at(Timestamp::now()));

        assert_eq!(
            provider.preflight_real_runtime().unwrap(),
            "fixture-runtime-1"
        );
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Initialized
        );
        assert!(provider
            .runtime_health()
            .ready_for_new_turn_at(Timestamp::now()));
        assert!(provider.runtime_health().observed_at >= first_observed);
        provider.shutdown_real_runtime().unwrap();
    }

    #[test]
    fn real_dsh_scope_admission_requires_exact_scope_identity() {
        let (provider, ctx, handle) = real_wire_fixture_provider();
        let mut missing = ctx.clone();
        missing.scope_id = None;
        assert!(provider.require_real_e0_scope(&missing).is_err());

        let mut foreign = ctx;
        foreign.scope_id = Some(format!("{}-other", handle.scope_id));
        assert!(provider.require_real_e0_scope(&foreign).is_err());
    }

    #[test]
    fn real_dsh_provider_completes_official_sdk_wire_contract_without_fake_lifecycle() {
        let (mut provider, ctx, handle) = real_wire_fixture_provider();
        let session = provider.start(&ctx).unwrap();
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Initialized
        );
        assert_eq!(
            provider.runtime_version().as_deref(),
            Some("fixture-runtime-1")
        );
        assert_eq!(provider.wire_server_version(), Some("0.0.1"));
        assert_eq!(
            provider.configured_runtime_digest(),
            Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );

        let output = provider.send(&session.id, "hello").unwrap();
        assert_eq!(output.text, "hello from fake sdk");
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Healthy
        );
        assert_eq!(provider.runtime_health().settled_turns, 1);
        assert!(provider.runtime_health().selectable_at(Timestamp::now()));

        let snapshot = provider.inspect(&session.id).unwrap();
        assert_eq!(snapshot.status, "idle");
        let events = provider.stream_events(&session.id);
        assert!(events
            .iter()
            .any(|event| event.kind == ExecutionEventKind::ModelResponse));
        assert!(events
            .iter()
            .any(|event| event.kind == ExecutionEventKind::Checkpoint));

        assert!(!provider.features().interrupt);
        assert!(!provider.features().resume);
        assert!(!provider.features().session_close);
        assert!(provider.interrupt(&session.id).is_err());
        assert!(provider.resume(&session.id).is_err());
        assert!(provider.terminate(&session.id).is_err());

        provider.unmount(&handle).unwrap();
        assert!(
            provider
                .send(&session.id, "must not run after scope revocation")
                .is_err(),
            "an unmounted E0 scope must revoke later prompt admission"
        );
        assert_eq!(provider.inspect(&session.id).unwrap().status, "idle");
        provider.shutdown_real_runtime().unwrap();
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Closed
        );
    }

    #[test]
    fn unsettled_real_dsh_turn_reaps_owned_runtime_before_returning() {
        use morn_kernel::ids::{ActorInstanceId, WorkPackageId};
        use morn_kernel::{ExecutionClass, ExecutionGuarantee};

        let executable = std::env::current_exe().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let home =
            std::env::temp_dir().join(format!("morn-dsh-timeout-home-{}", uuid::Uuid::new_v4()));
        let environment_ref = "env://container/dsh-timeout-test";
        let mut config =
            DshSdkConfig::profile_sdk(cwd.to_string_lossy(), "fixture-provider", "fixture-model")
                .with_dsh_home(home.to_string_lossy())
                .with_execution_environment_ref(environment_ref)
                .with_runtime_identity(
                    "fixture-runtime-1",
                    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                );
        config.command = executable.to_string_lossy().to_string();
        config.args = vec![
            "--exact".to_string(),
            "dsh_sdk::tests::fake_sdk_runtime".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
        ];
        config.allow_protocol_fixture_transport();
        config.request_timeout_ms = 2_000;
        config.turn_timeout_ms = 100;

        let workspace = WorkspaceId::generate();
        let mut provider = DeepSeekHarnessProvider::with_real_sdk(config);
        let scope = CapabilityScope::new(
            crate::scope::ScopeKind::ExecutionRun,
            None,
            workspace.clone(),
            "real-dsh-timeout",
        )
        .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
        let handle = provider.mount(scope).unwrap();
        let ctx = RuntimeContext::new(
            workspace,
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        )
        .with_work_binding(
            1,
            morn_kernel::ids::RuntimeBindingId::generate_with("binding"),
        )
        .unwrap()
        .with_scope_id(handle.scope_id)
        .unwrap()
        .with_execution_environment(
            environment_ref,
            ExecutionClass::Container,
            vec![
                ExecutionGuarantee::FilesystemReadPolicy,
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::ProcessBoundary,
                ExecutionGuarantee::ResourceLimits,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
                ExecutionGuarantee::RuntimeAttestation,
                ExecutionGuarantee::ToolMediation,
            ],
        )
        .unwrap();

        let session = provider.start(&ctx).unwrap();
        let error = provider.send(&session.id, "__hang__").unwrap_err();
        assert!(format!("{error}").contains("timed out"));
        assert!(provider.real_client.is_none());
        assert!(provider.runtime_version().is_none());
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Degraded
        );
        assert_eq!(
            provider.inspect(&session.id).unwrap().status,
            "outcome-unknown-runtime-reaped"
        );
        assert!(provider.send(&session.id, "do not retry").is_err());
    }

    #[test]
    fn non_successful_real_dsh_turn_never_refreshes_provider_health() {
        let (mut provider, ctx, _handle) = real_wire_fixture_provider();
        let session = provider.start(&ctx).unwrap();
        assert!(provider.send(&session.id, "__non_success__").is_err());
        assert_eq!(
            provider.runtime_health().state,
            HarnessRuntimeHealthState::Degraded
        );
        assert_eq!(provider.runtime_health().settled_turns, 0);
        assert!(!provider.runtime_health().selectable_at(Timestamp::now()));
        assert_eq!(
            provider.inspect(&session.id).unwrap().status,
            "idle-non-success"
        );
        provider.shutdown_real_runtime().unwrap();
    }

    #[test]
    fn runtime_health_requires_a_fresh_live_turn_before_selection() {
        let mut health = HarnessRuntimeHealth::configured("configured");
        let configured_at = health.observed_at;
        assert!(!health.selectable_at(configured_at));

        health.mark_initialized("initialized", "runtime://dsh/init");
        assert_eq!(health.state, HarnessRuntimeHealthState::Initialized);
        assert!(!health.selectable_at(health.observed_at));
        assert!(health.ready_for_new_turn_at(health.observed_at));
        let stale_initialization = Timestamp::from_millis(
            health
                .observed_at
                .millis()
                .saturating_add(HARNESS_RUNTIME_HEALTH_LEASE_MS + 1),
        );
        assert!(!health.ready_for_new_turn_at(stale_initialization));

        health.mark_live_turn("settled", "runtime://dsh/turn");
        let observed = health.observed_at;
        assert!(health.selectable_at(observed));
        assert_eq!(health.settled_turns, 1);
        assert!(health.remaining_lease_ms(observed).is_some());

        let expired = Timestamp::from_millis(
            observed
                .millis()
                .saturating_add(HARNESS_RUNTIME_HEALTH_LEASE_MS + 1),
        );
        assert!(!health.selectable_at(expired));
        assert!(health.remaining_lease_ms(expired).is_none());

        assert_eq!(health.evidence_refs, vec!["runtime://dsh/turn".to_string()]);
        health.mark_degraded("transport failed");
        assert_eq!(health.state, HarnessRuntimeHealthState::Degraded);
        assert!(health.evidence_refs.is_empty());
        assert!(!health.selectable_at(Timestamp::now()));

        health.mark_initialized("restarted", "runtime://dsh/restarted");
        assert_eq!(
            health.evidence_refs,
            vec!["runtime://dsh/restarted".to_string()]
        );
        health.mark_closed("runtime closed");
        assert_eq!(health.state, HarnessRuntimeHealthState::Closed);
        assert!(health.evidence_refs.is_empty());
    }

    #[test]
    fn e0_policy_fails_closed_for_unknown_tool_lifecycle_and_subagents() {
        for notification in [
            DshNotification {
                method: DSH_NOTIFICATION_SESSION_EVENT.to_string(),
                params: json!({
                    "sessionId":"s1",
                    "event":{"type":"tool/future-lifecycle","data":{}}
                }),
            },
            DshNotification {
                method: DSH_NOTIFICATION_SUBAGENT_STARTED.to_string(),
                params: json!({"parentSessionId":"s1","childSessionId":"child-1"}),
            },
            DshNotification {
                method: DSH_NOTIFICATION_SUBAGENT_FINISHED.to_string(),
                params: json!({
                    "provider":"local",
                    "agentId":"child-1",
                    "parentSessionId":"s1",
                    "childSessionId":"child-1",
                    "status":"ok",
                    "stopReason":"completed"
                }),
            },
        ] {
            assert!(
                dsh_notification_violates_e0(&notification),
                "new tool/subagent surfaces must fail closed on the E0 provider path"
            );
        }

        let workspace = WorkspaceId::generate();
        let unknown = DshNotification {
            method: DSH_NOTIFICATION_SESSION_EVENT.to_string(),
            params: json!({
                "sessionId":"s1",
                "event":{"type":"tool/future-lifecycle","data":{}}
            }),
        };
        let normalized = normalize_dsh_notifications(&workspace, "s1", &[unknown]);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].kind, ExecutionEventKind::ToolProposed);
    }

    #[test]
    fn normalizes_durable_dsh_events_without_copying_content() {
        let workspace = WorkspaceId::generate();
        let notifications = vec![
            DshNotification {
                method: "session.event".to_string(),
                params: json!({
                    "sessionId":"s1",
                    "event":{"type":"request/header","data":{"header":{"config":{"model":"deepseek-v4"}}}}
                }),
            },
            DshNotification {
                method: "session.event".to_string(),
                params: json!({
                    "sessionId":"s1",
                    "event":{"type":"assistant/message","data":{"message":{"content":[{"type":"text","text":"private output body"}]}}}
                }),
            },
            DshNotification {
                method: "session.event".to_string(),
                params: json!({
                    "sessionId":"s1",
                    "event":{"type":"tool/call","data":{"callId":"call-1","name":"shell","arguments":"secret args"}}
                }),
            },
            DshNotification {
                method: "session.event".to_string(),
                params: json!({
                    "sessionId":"s1",
                    "event":{"type":"tool/result","data":{"message":{"toolCallId":"call-1","isError":false,"content":[{"type":"text","text":"secret result"}]}}}
                }),
            },
        ];
        let events = normalize_dsh_notifications(&workspace, "s1", &notifications);
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].kind, ExecutionEventKind::ModelRequest);
        assert_eq!(events[0].model_version.as_deref(), Some("deepseek-v4"));
        assert_eq!(events[1].kind, ExecutionEventKind::ModelResponse);
        assert_eq!(events[2].kind, ExecutionEventKind::ToolProposed);
        assert_eq!(events[3].kind, ExecutionEventKind::ToolCompleted);
        for event in &events {
            assert!(!event.summary.contains("private output body"));
            assert!(!event.summary.contains("secret args"));
            assert!(!event.summary.contains("secret result"));
        }
        assert!(events[2].refs.contains(&"dsh-tool-call:call-1".to_string()));
    }
}
