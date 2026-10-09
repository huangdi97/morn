//! Lightweight Pi harness provider.
//!
//! Pi is treated as a replaceable harness provider. This reference adapter is
//! deliberately transport-neutral: fixture mode proves Morn's harness contract
//! without letting a Pi session become canonical Work truth. A real transport
//! can be added behind the same trait without changing Morn semantics.

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::ExecutionReceiptId;

use crate::context::RuntimeContext;
use crate::event::{ExecutionEvent, ExecutionEventKind};
use crate::pi_rpc::{PiRpcClient, PiRpcConfig};
use crate::provider::{
    HarnessOutput, HarnessProvider, HarnessProviderFeatures, HarnessRuntimeHealth, HarnessSession,
    HarnessSnapshot, ProviderHandle,
};
use crate::receipt::ExecutionReceipt;
use crate::scope::CapabilityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiMode {
    Fixture,
    Real,
}

pub const PI_REAL_E0_SCOPE_RESTRICTION: &str = "morn.effects<=E0";

#[derive(Debug)]
struct PiSessionState {
    ctx: RuntimeContext,
    status: String,
    step: u64,
    events: Vec<ExecutionEvent>,
    last_event: String,
}

#[derive(Debug)]
pub struct PiHarnessProvider {
    name: String,
    mode: PiMode,
    sessions: HashMap<String, PiSessionState>,
    scopes: Vec<CapabilityScope>,
    real_config: Option<PiRpcConfig>,
    real_client: Option<PiRpcClient>,
    active_session: Option<String>,
    runtime_health: HarnessRuntimeHealth,
}

impl PiHarnessProvider {
    pub fn new(mode: PiMode) -> Self {
        let runtime_health = match mode {
            PiMode::Fixture => HarnessRuntimeHealth::fixture(
                "deterministic fixture contract; not a live external runtime",
            ),
            PiMode::Real => HarnessRuntimeHealth::unconfigured(
                "real Pi mode selected without an explicit RPC configuration",
            ),
        };
        Self {
            name: "pi".to_string(),
            mode,
            sessions: HashMap::new(),
            scopes: Vec::new(),
            real_config: None,
            real_client: None,
            active_session: None,
            runtime_health,
        }
    }

    pub fn with_real_rpc(config: PiRpcConfig) -> Self {
        let mut provider = Self::new(PiMode::Real);
        provider.real_config = Some(config);
        provider.runtime_health = HarnessRuntimeHealth::configured(
            "real Pi RPC configuration accepted; live handshake not yet performed",
        );
        provider
    }

    pub fn from_real_env() -> Result<Self> {
        let config = PiRpcConfig::from_env()?;
        Ok(Self::with_real_rpc(config))
    }

    pub fn mode(&self) -> PiMode {
        self.mode
    }

    pub fn runtime_health(&self) -> &HarnessRuntimeHealth {
        &self.runtime_health
    }

    pub fn shutdown_real_runtime(&mut self) -> Result<()> {
        if self.mode != PiMode::Real {
            return Err(Error::invalid_state(
                "only a real Pi provider owns an RPC runtime",
            ));
        }
        if let Some(mut client) = self.real_client.take() {
            if let Err(error) = client.shutdown() {
                self.runtime_health
                    .mark_degraded(format!("Pi RPC shutdown failed: {error}"));
                return Err(error);
            }
        }
        self.runtime_health
            .mark_closed("Pi RPC runtime was explicitly shut down");
        for state in self.sessions.values_mut() {
            if state.status != "terminated" {
                state.status = "runtime-closed".to_string();
                state.last_event = "runtime_closed".to_string();
            }
        }
        self.active_session = None;
        Ok(())
    }

    fn ensure_real_client(&mut self) -> Result<&mut PiRpcClient> {
        if self.real_client.is_none() {
            let setup = (|| -> Result<PiRpcClient> {
                let config = self
                    .real_config
                    .clone()
                    .ok_or_else(|| self.real_unavailable())?;
                let mut client = PiRpcClient::spawn(&config)?;
                client.get_state()?;
                Ok(client)
            })();
            match setup {
                Ok(client) => {
                    self.real_client = Some(client);
                    self.runtime_health.mark_initialized(
                        "Pi RPC runtime handshake succeeded; no settled live turn yet",
                        "runtime://pi/get-state",
                    );
                }
                Err(error) => {
                    self.runtime_health
                        .mark_degraded(format!("Pi RPC startup/get_state failed: {error}"));
                    return Err(error);
                }
            }
        }
        self.real_client
            .as_mut()
            .ok_or_else(|| Error::internal("Pi RPC client missing after startup"))
    }

    fn real_unavailable(&self) -> Error {
        Error::external(
            "Pi real RPC transport is not configured; construct PiHarnessProvider::with_real_rpc with an explicit runtime config",
        )
    }

    fn require_real_e0_scope(&self, ctx: &RuntimeContext) -> Result<()> {
        let eligible = self.scopes.iter().any(|scope| {
            scope.workspace_id == ctx.workspace_id
                && scope
                    .restrictions
                    .iter()
                    .any(|restriction| restriction == PI_REAL_E0_SCOPE_RESTRICTION)
                && ctx
                    .scope_id
                    .as_deref()
                    .is_none_or(|scope_id| scope.id.as_str() == scope_id)
        });
        if eligible {
            Ok(())
        } else {
            Err(Error::validation(
                "real Pi RPC requires a matching isolated E0 scope; E1/E2/E3 actions must use Morn ExternalAction",
            ))
        }
    }

    fn require_pinned_real_environment(&self, ctx: &RuntimeContext) -> Result<()> {
        if !ctx.proves_real_harness_environment() {
            return Err(Error::validation(
                "real Pi RPC requires a runtime-attested container-or-stronger environment with read/write policy, process boundary, resource limits, network-egress and secret indirection",
            ));
        }
        let configured = self
            .real_config
            .as_ref()
            .and_then(|config| config.execution_environment_ref.as_deref())
            .filter(|reference| !reference.trim().is_empty())
            .ok_or_else(|| {
                Error::validation("real Pi RPC configuration is missing execution_environment_ref")
            })?;
        if ctx.execution_environment_ref.as_deref() != Some(configured) {
            return Err(Error::validation(
                "real Pi RPC RuntimeContext execution environment does not match the pinned launch environment",
            ));
        }
        Ok(())
    }
}

impl HarnessProvider for PiHarnessProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn features(&self) -> HarnessProviderFeatures {
        match self.mode {
            PiMode::Fixture => HarnessProviderFeatures::full_reference(),
            PiMode::Real => HarnessProviderFeatures::pi_rpc_current(),
        }
    }

    fn mount(&mut self, scope: CapabilityScope) -> Result<ProviderHandle> {
        if self.mode == PiMode::Real
            && !scope
                .restrictions
                .iter()
                .any(|restriction| restriction == PI_REAL_E0_SCOPE_RESTRICTION)
        {
            return Err(Error::validation(
                "real Pi RPC scope must explicitly declare morn.effects<=E0",
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
        self.scopes
            .retain(|scope| scope.id.to_string() != handle.scope_id);
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
            PiMode::Fixture => {
                let session_id = format!("pi-fixture-{}", uuid::Uuid::new_v4());
                let event = ExecutionEvent::new(
                    ctx.workspace_id.clone(),
                    session_id.clone(),
                    ExecutionEventKind::SessionStarted,
                    "pi fixture session started",
                );
                self.sessions.insert(
                    session_id.clone(),
                    PiSessionState {
                        ctx: ctx.clone(),
                        status: "running".to_string(),
                        step: 0,
                        events: vec![event],
                        last_event: "session_started".to_string(),
                    },
                );
                Ok(HarnessSession {
                    id: session_id,
                    provider: self.name.clone(),
                    status: "running".to_string(),
                })
            }
            PiMode::Real => {
                if self.real_config.is_none() {
                    return Err(self.real_unavailable());
                }
                self.require_real_e0_scope(ctx)?;
                self.require_pinned_real_environment(ctx)?;
                if let Some(active) = self.active_session.as_ref() {
                    if self
                        .sessions
                        .get(active)
                        .is_some_and(|state| state.status != "runtime-closed")
                    {
                        return Err(Error::conflict(format!(
                            "Pi RPC provider already owns active session {active}"
                        )));
                    }
                }
                self.ensure_real_client()?;
                let session_id = format!("morn-pi-{}", uuid::Uuid::new_v4());
                let event = ExecutionEvent::new(
                    ctx.workspace_id.clone(),
                    session_id.clone(),
                    ExecutionEventKind::SessionStarted,
                    "Pi RPC runtime ready; one ephemeral provider session reserved",
                );
                self.sessions.insert(
                    session_id.clone(),
                    PiSessionState {
                        ctx: ctx.clone(),
                        status: "ready".to_string(),
                        step: 0,
                        events: vec![event],
                        last_event: "rpc_runtime_ready".to_string(),
                    },
                );
                self.active_session = Some(session_id.clone());
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
            PiMode::Fixture => {
                let state = self
                    .sessions
                    .get_mut(session_id)
                    .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                state.step += 1;
                let step = state.step;
                state.events.push(ExecutionEvent::new(
                    state.ctx.workspace_id.clone(),
                    session_id.to_string(),
                    ExecutionEventKind::ModelResponse,
                    format!("pi fixture response for step {step}"),
                ));
                state.last_event = format!("model_response:{step}");
                Ok(HarnessOutput {
                    session_id: session_id.to_string(),
                    text: format!("pi fixture output for {input:?}"),
                    proposal_ref: None,
                })
            }
            PiMode::Real => {
                if input.trim().is_empty() {
                    return Err(Error::validation("Pi RPC prompt must be non-empty"));
                }
                if self.active_session.as_deref() != Some(session_id) {
                    return Err(Error::invalid_state(
                        "Pi RPC prompt must target the provider's active ephemeral session",
                    ));
                }
                {
                    let state = self
                        .sessions
                        .get_mut(session_id)
                        .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                    if !matches!(state.status.as_str(), "ready" | "idle") {
                        return Err(Error::invalid_state(format!(
                            "Pi session {session_id} cannot accept a new prompt from state {}",
                            state.status
                        )));
                    }
                    state.step += 1;
                    state.status = "running".to_string();
                    state.events.push(ExecutionEvent::new(
                        state.ctx.workspace_id.clone(),
                        session_id.to_string(),
                        ExecutionEventKind::ModelRequest,
                        format!("Pi RPC prompt admitted for step {}", state.step),
                    ));
                    state.last_event = format!("pi_prompt:{}", state.step);
                }

                let run = self.ensure_real_client()?.prompt_and_wait(input);
                if let Ok(settled) = &run {
                    self.runtime_health.mark_live_turn(
                        format!(
                            "live Pi RPC turn settled with disposition {}",
                            settled.disposition
                        ),
                        format!("runtime://pi/session/{session_id}/settled-turn"),
                    );
                } else if let Err(error) = &run {
                    self.runtime_health
                        .mark_degraded(format!("Pi RPC turn failed to settle: {error}"));
                }
                match run {
                    Ok(run) => {
                        let text = self
                            .ensure_real_client()?
                            .get_last_assistant_text()?
                            .unwrap_or_default();
                        let state = self
                            .sessions
                            .get_mut(session_id)
                            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                        state.status = "idle".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::ModelResponse,
                            format!(
                                "Pi RPC prompt settled; request={} disposition={} events={}",
                                run.request_id,
                                run.disposition,
                                run.events.len()
                            ),
                        ));
                        state.last_event = "pi_agent_settled".to_string();
                        Ok(HarnessOutput {
                            session_id: session_id.to_string(),
                            text,
                            proposal_ref: None,
                        })
                    }
                    Err(error) => {
                        let state = self
                            .sessions
                            .get_mut(session_id)
                            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
                        state.status = "outcome-unknown".to_string();
                        state.events.push(ExecutionEvent::new(
                            state.ctx.workspace_id.clone(),
                            session_id.to_string(),
                            ExecutionEventKind::Failed,
                            "Pi RPC turn lost definitive settlement; blind retry is forbidden",
                        ));
                        state.last_event = "pi_turn_outcome_unknown".to_string();
                        Err(error)
                    }
                }
            }
        }
    }

    fn stream_events(&self, session_id: &str) -> Vec<ExecutionEvent> {
        self.sessions
            .get(session_id)
            .map(|state| state.events.clone())
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
        if self.mode == PiMode::Real {
            if self.active_session.as_deref() != Some(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            self.ensure_real_client()?.abort()?;
            let state = self
                .sessions
                .get_mut(session_id)
                .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
            let ambiguous = state.status == "outcome-unknown";
            state.status = if ambiguous {
                "outcome-unknown".to_string()
            } else {
                "idle".to_string()
            };
            state.events.push(ExecutionEvent::new(
                state.ctx.workspace_id.clone(),
                session_id.to_string(),
                ExecutionEventKind::Interrupted,
                if ambiguous {
                    "Pi RPC abort stopped executor activity; prior external effect remains outcome-unknown"
                } else {
                    "Pi RPC abort completed and agent returned idle"
                },
            ));
            state.last_event = if ambiguous {
                "aborted_requires_reconciliation".to_string()
            } else {
                "aborted".to_string()
            };
            return Ok(());
        }
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        state.status = "paused".to_string();
        state.events.push(ExecutionEvent::new(
            state.ctx.workspace_id.clone(),
            session_id.to_string(),
            ExecutionEventKind::Interrupted,
            "pi fixture session interrupted",
        ));
        state.last_event = "interrupted".to_string();
        Ok(())
    }

    fn resume(&mut self, session_id: &str) -> Result<()> {
        if self.mode == PiMode::Real {
            if !self.sessions.contains_key(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            return Err(Error::invalid_state(
                "Pi RPC --no-session does not support cross-process session resume",
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
        state.events.push(ExecutionEvent::new(
            state.ctx.workspace_id.clone(),
            session_id.to_string(),
            ExecutionEventKind::Resumed,
            "pi fixture session resumed",
        ));
        state.last_event = "resumed".to_string();
        Ok(())
    }

    fn terminate(&mut self, session_id: &str) -> Result<ExecutionReceipt> {
        if self.mode == PiMode::Real {
            if !self.sessions.contains_key(session_id) {
                return Err(Error::not_found(format!("session {session_id}")));
            }
            return Err(Error::invalid_state(
                "Pi RPC has process-level shutdown rather than per-session close; call shutdown_real_runtime",
            ));
        }
        let state = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("session {session_id}")))?;
        state.status = "terminated".to_string();
        state.events.push(ExecutionEvent::new(
            state.ctx.workspace_id.clone(),
            session_id.to_string(),
            ExecutionEventKind::Completed,
            "pi fixture session completed",
        ));
        state.last_event = "completed".to_string();
        let event_ids: Vec<String> = state
            .events
            .iter()
            .map(|event| event.id.to_string())
            .collect();
        Ok(ExecutionReceipt {
            id: ExecutionReceiptId::generate_with("rcpt"),
            workspace_id: state.ctx.workspace_id.clone(),
            session_id: session_id.to_string(),
            trace_refs: event_ids.clone(),
            started_at: state
                .events
                .first()
                .map(|event| event.created_at)
                .unwrap_or_default(),
            ended_at: Some(morn_kernel::time::Timestamp::now()),
            outcome: "completed".to_string(),
            harness_version: None,
            runtime_version: Some("pi-fixture".to_string()),
            event_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::run_provider_contract;
    use morn_kernel::ids::{ActorInstanceId, WorkPackageId, WorkspaceId};

    #[test]
    fn real_pi_rejects_context_from_a_different_execution_environment() {
        use morn_kernel::{ExecutionClass, ExecutionGuarantee};

        let config = PiRpcConfig {
            cwd: Some(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ),
            provider: Some("fixture-provider".to_string()),
            model: Some("fixture-model".to_string()),
            execution_environment_ref: Some("env://container/pinned".to_string()),
            ..Default::default()
        };
        let provider = PiHarnessProvider::with_real_rpc(config);
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

    #[test]
    fn pi_runtime_health_does_not_claim_live_before_transport_evidence() {
        let fixture = PiHarnessProvider::new(PiMode::Fixture);
        assert_eq!(
            fixture.runtime_health().state,
            crate::provider::HarnessRuntimeHealthState::Fixture
        );

        let real = PiHarnessProvider::new(PiMode::Real);
        assert_eq!(
            real.runtime_health().state,
            crate::provider::HarnessRuntimeHealthState::Unconfigured
        );
        assert!(!real
            .runtime_health()
            .selectable_at(morn_kernel::time::Timestamp::now()));
    }

    #[test]
    fn pi_real_features_match_rpc_no_session_contract() {
        let provider = PiHarnessProvider::new(PiMode::Real);
        let features = provider.features();
        assert!(features.interrupt);
        assert!(!features.resume);
        assert!(!features.session_close);
        assert!(!features.durable_events);
        assert!(!features.multi_session);
    }

    #[test]
    fn pi_fixture_passes_shared_harness_contract() {
        let ctx = RuntimeContext::new(
            WorkspaceId::generate(),
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        );
        let mut provider = PiHarnessProvider::new(PiMode::Fixture);
        let report = run_provider_contract(&mut provider, &ctx).unwrap();
        assert!(report.all_passed(), "{:?}", report.checks);
    }
}
