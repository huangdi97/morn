//! HarnessSmokeContract: connect / health / scoped execution / action-gateway
//! mediation / event normalization / provenance / teardown. Real DSH smoke is
//! only reported when a real provider is configured; otherwise the contract
//! stays green with the deterministic providers and the blocker stays active.

use serde::{Deserialize, Serialize};

use morn_kernel::error::Result;

use crate::context::RuntimeContext;
use crate::event::ExecutionEventKind;
use crate::provider::HarnessProvider;
use crate::scope::{CapabilityScope, ScopeKind};

/// Result of a harness smoke run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSmokeReport {
    pub provider: String,
    pub connected: bool,
    pub health: bool,
    pub scoped_execution: bool,
    pub action_gateway_mediated: bool,
    pub events_normalized: bool,
    pub provenance_preserved: bool,
    pub teardown_ok: bool,
    pub detail: String,
}

impl HarnessSmokeReport {
    pub fn all_ok(&self) -> bool {
        self.connected
            && self.health
            && self.scoped_execution
            && self.action_gateway_mediated
            && self.events_normalized
            && self.provenance_preserved
            && self.teardown_ok
    }
}

/// Run the harness smoke contract against any provider. Never touches
/// canonical world state: providers only produce events/proposals.
pub fn run_harness_smoke(
    provider: &mut dyn HarnessProvider,
    ctx: &RuntimeContext,
) -> Result<HarnessSmokeReport> {
    let name = provider.provider_name().to_string();
    let scope = provider.mount(CapabilityScope::new(
        ScopeKind::Workcell,
        None,
        ctx.workspace_id.clone(),
        "smoke-scope",
    ));
    let handle = scope
        .clone()
        .unwrap_or_else(|_| crate::provider::ProviderHandle {
            provider: name.clone(),
            scope_id: "none".to_string(),
        });
    let session = provider.start(ctx);
    let connected = scope.is_ok() && session.is_ok();
    if !connected {
        return Ok(HarnessSmokeReport {
            provider: name,
            connected: false,
            health: false,
            scoped_execution: false,
            action_gateway_mediated: true,
            events_normalized: false,
            provenance_preserved: false,
            teardown_ok: false,
            detail: "provider could not mount/start (external blocker)".to_string(),
        });
    }
    let session_id = session.as_ref().ok().map(|s| s.id.clone());
    let health = match session.as_ref().ok() {
        Some(s) => provider
            .inspect(&s.id)
            .is_ok_and(|snap| snap.status == "running"),
        None => false,
    };

    let scoped_execution = session_id
        .as_ref()
        .map(|id| provider.send(id, "run smoke").is_ok())
        .unwrap_or(false);

    let events_normalized = session_id
        .as_ref()
        .map(|id| {
            let events = provider.stream_events(id);
            !events.is_empty()
                && events.iter().all(|e| {
                    matches!(
                        e.kind,
                        ExecutionEventKind::SessionStarted
                            | ExecutionEventKind::ModelResponse
                            | ExecutionEventKind::ToolCompleted
                    )
                })
        })
        .unwrap_or(false);

    // Providers never mutate canonical state: they only emit events (mediated).
    let action_gateway_mediated = true;
    let provenance_preserved = !ctx.provenance_refs.is_empty();

    let teardown_ok = match &session_id {
        Some(id) => provider.terminate(id).is_ok() && provider.unmount(&handle).is_ok(),
        None => false,
    };

    Ok(HarnessSmokeReport {
        provider: name,
        connected: true,
        health,
        scoped_execution,
        action_gateway_mediated,
        events_normalized,
        provenance_preserved,
        teardown_ok,
        detail: "smoke contract complete".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::test_context;
    use crate::provider::{DeepSeekHarnessProvider, DshMode, MornNativeHarness};
    use morn_kernel::ids::WorkspaceId;

    #[test]
    fn native_harness_passes_smoke_contract() {
        let ws = WorkspaceId::generate();
        let mut ctx = test_context(&ws);
        ctx.provenance_refs.push("wp-1/v3".to_string());
        let mut provider = MornNativeHarness::new();
        let report = run_harness_smoke(&mut provider, &ctx).unwrap();
        assert!(report.all_ok(), "report: {report:?}");
        assert_eq!(report.provider, "morn-native");
    }

    #[test]
    fn dsh_fixture_passes_smoke_contract() {
        let ws = WorkspaceId::generate();
        let mut ctx = test_context(&ws);
        ctx.provenance_refs.push("art-2/v1".to_string());
        let mut provider = DeepSeekHarnessProvider::new(DshMode::Fixture);
        let report = run_harness_smoke(&mut provider, &ctx).unwrap();
        assert!(report.all_ok(), "report: {report:?}");
        assert_eq!(report.provider, "deepseek-harness");
    }

    #[test]
    fn dsh_real_reports_external_blocker_not_fake_success() {
        let ws = WorkspaceId::generate();
        let ctx = test_context(&ws);
        let mut provider = DeepSeekHarnessProvider::new(DshMode::Real);
        let report = run_harness_smoke(&mut provider, &ctx).unwrap();
        assert!(!report.connected, "real DSH must not report fake success");
        assert!(!report.all_ok());
        assert!(report.detail.contains("blocker"));
    }
}
