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
use crate::provider::{
    HarnessOutput, HarnessProvider, HarnessSession, HarnessSnapshot, ProviderHandle,
};
use crate::receipt::ExecutionReceipt;
use crate::scope::CapabilityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiMode {
    Fixture,
    Real,
}

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
}

impl PiHarnessProvider {
    pub fn new(mode: PiMode) -> Self {
        Self {
            name: "pi".to_string(),
            mode,
            sessions: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn mode(&self) -> PiMode {
        self.mode
    }

    fn real_unavailable(&self) -> Error {
        Error::external(
            "Pi real transport is not configured in this reference runtime;              use fixture mode for contract tests or provide an external Pi transport adapter",
        )
    }
}

impl HarnessProvider for PiHarnessProvider {
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
            PiMode::Real => Err(self.real_unavailable()),
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
            PiMode::Real => Err(self.real_unavailable()),
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
    fn pi_fixture_passes_shared_harness_contract() {
        let ctx = RuntimeContext {
            workspace_id: WorkspaceId::generate(),
            work_package_id: WorkPackageId::generate_with("work"),
            actor_id: ActorInstanceId::generate_with("actor"),
            correlation_id: "pi-contract".to_string(),
            trace_id: "trace-pi-contract".to_string(),
        };
        let mut provider = PiHarnessProvider::new(PiMode::Fixture);
        let report = run_provider_contract(&mut provider, &ctx).unwrap();
        assert!(report.all_passed(), "{:?}", report.checks);
    }
}
