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
    let mut smoke_scope = CapabilityScope::new(
        ScopeKind::Workcell,
        None,
        ctx.workspace_id.clone(),
        "smoke-scope",
    );
    for restriction in provider.required_scope_restrictions() {
        smoke_scope = smoke_scope.with_restriction(*restriction);
    }
    let scope = provider.mount(smoke_scope);
    let handle = scope
        .clone()
        .unwrap_or_else(|_| crate::provider::ProviderHandle {
            provider: name.clone(),
            scope_id: "none".to_string(),
        });
    let scoped_ctx = match ctx.clone().with_scope_id(handle.scope_id.clone()) {
        Ok(context) => context,
        Err(error) => {
            let cleanup_ok = provider.unmount(&handle).is_ok();
            return Ok(HarnessSmokeReport {
                provider: name,
                connected: false,
                health: false,
                scoped_execution: false,
                action_gateway_mediated: true,
                events_normalized: false,
                provenance_preserved: false,
                teardown_ok: cleanup_ok,
                detail: format!("cannot bind smoke scope to RuntimeContext: {error}"),
            });
        }
    };
    let session = provider.start(&scoped_ctx);
    let connected = scope.is_ok() && session.is_ok();
    if !connected {
        let cleanup_ok = if scope.is_ok() {
            provider.unmount(&handle).is_ok()
        } else {
            true
        };
        return Ok(HarnessSmokeReport {
            provider: name,
            connected: false,
            health: false,
            scoped_execution: false,
            action_gateway_mediated: true,
            events_normalized: false,
            provenance_preserved: false,
            teardown_ok: cleanup_ok,
            detail: "provider could not mount/start (external blocker); any mounted scope was cleaned up".to_string(),
        });
    }
    let session_id = session.as_ref().ok().map(|s| s.id.clone());
    let health = match session.as_ref().ok() {
        Some(session) => provider.inspect(&session.id).is_ok_and(|snapshot| {
            snapshot.session_id == session.id && !snapshot.status.trim().is_empty()
        }),
        None => false,
    };

    let governed_external_runtime = !provider.required_scope_restrictions().is_empty();
    let scoped_execution = session_id
        .as_ref()
        .map(|id| {
            provider
                .send(
                    id,
                    "Return exactly MORN_PROVIDER_SMOKE_OK. Do not call tools, access files, use the network, or modify any external system.",
                )
                .is_ok()
        })
        .unwrap_or(false);

    let events = session_id
        .as_ref()
        .map(|id| provider.stream_events(id))
        .unwrap_or_default();
    let events_normalized = session_id.as_ref().is_some_and(|id| {
        !events.is_empty()
            && events.iter().all(|event| {
                event.workspace_id == scoped_ctx.workspace_id
                    && event.session_id.as_str() == id.as_str()
                    && !event.summary.trim().is_empty()
            })
    });
    let tool_activity = events.iter().any(|event| {
        matches!(
            event.kind,
            ExecutionEventKind::ToolProposed
                | ExecutionEventKind::ToolStarted
                | ExecutionEventKind::ToolCompleted
                | ExecutionEventKind::ToolFailed
        )
    });

    // Real external harnesses are admitted only as E0 executors. During this
    // live smoke they receive an explicit no-tool prompt; any tool lifecycle
    // event therefore proves the provider escaped the intended mediation path.
    let action_gateway_mediated = !governed_external_runtime || !tool_activity;
    let provenance_preserved = !scoped_ctx.provenance_refs.is_empty();

    let features = provider.features();
    let teardown_ok = match &session_id {
        Some(id) => {
            let session_cleanup = if features.session_close {
                provider.terminate(id).is_ok()
            } else {
                true
            };
            session_cleanup && provider.unmount(&handle).is_ok()
        }
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
        detail: if governed_external_runtime && tool_activity {
            "smoke observed tool activity despite an explicit E0/no-tool probe".to_string()
        } else if features.session_close {
            "smoke contract complete".to_string()
        } else {
            "smoke contract complete; per-session close is unsupported and was not fabricated"
                .to_string()
        },
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
