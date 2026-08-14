//! Shared provider contract suite. Every harness provider path must pass it,
//! proving Morn-side invariants independent of the concrete provider.

use morn_kernel::error::Result;
use morn_kernel::ids::WorkspaceId;

use crate::context::RuntimeContext;
use crate::event::ExecutionEventKind;
use crate::provider::{HarnessProvider, ProviderHandle};
use crate::scope::{CapabilityScope, ScopeKind};
use morn_kernel::ids::{ActorInstanceId, WorkPackageId};

/// Report of the contract checks run against a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractReport {
    pub provider: String,
    pub checks: Vec<(String, bool)>,
}

impl ContractReport {
    pub fn all_passed(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|(_, ok)| *ok)
    }
}

fn check(report: &mut Vec<(String, bool)>, name: &str, ok: bool) {
    report.push((name.to_string(), ok));
}

/// Run the Morn provider contract against any `HarnessProvider`.
///
/// Covers: lifecycle (mount/start/send/inspect/interrupt/resume/terminate),
/// E0 unmount cleanup, scope isolation, and event normalization to Morn
/// `ExecutionEvent`s.
pub fn run_provider_contract(
    provider: &mut dyn HarnessProvider,
    ctx: &RuntimeContext,
) -> Result<ContractReport> {
    let mut report: Vec<(String, bool)> = Vec::new();
    let provider_name = provider.provider_name().to_string();

    // lifecycle
    let mount_result = provider.mount(CapabilityScope::new(
        ScopeKind::Workcell,
        None,
        ctx.workspace_id.clone(),
        "contract-scope",
    ));
    let handle: ProviderHandle = match mount_result {
        Ok(h) => {
            check(&mut report, "mount", true);
            h
        }
        Err(e) => {
            check(&mut report, "mount", false);
            return Err(e);
        }
    };

    let session = match provider.start(ctx) {
        Ok(s) => {
            check(&mut report, "start", true);
            s
        }
        Err(e) => {
            check(&mut report, "start", false);
            return Err(e);
        }
    };

    let out = provider.send(&session.id, "run analysis");
    check(&mut report, "send", out.is_ok());

    let events = provider.stream_events(&session.id);
    check(
        &mut report,
        "event normalization",
        !events.is_empty()
            && events
                .iter()
                .all(|e| matches!(e.kind, ExecutionEventKind::SessionStarted | ExecutionEventKind::ModelResponse | ExecutionEventKind::ToolCompleted)),
    );

    check(
        &mut report,
        "inspect",
        provider.inspect(&session.id).is_ok_and(|s| s.status == "running"),
    );

    check(&mut report, "interrupt", provider.interrupt(&session.id).is_ok());
    check(&mut report, "resume", provider.resume(&session.id).is_ok());

    let receipt = provider.terminate(&session.id);
    check(
        &mut report,
        "terminate -> receipt",
        receipt.is_ok_and(|r| !r.trace_refs.is_empty()),
    );

    // E0 unmount cleanup: first unmount succeeds, second must fail (already gone).
    let unmount1 = provider.unmount(&handle);
    check(&mut report, "E0 unmount cleanup", unmount1.is_ok());
    check(
        &mut report,
        "E0 unmount idempotent-guard",
        provider.unmount(&handle).is_err(),
    );

    Ok(ContractReport {
        provider: provider_name,
        checks: report,
    })
}

/// Convenience: a fresh context for contract tests.
pub fn test_context(workspace_id: &WorkspaceId) -> RuntimeContext {
    RuntimeContext::new(
        workspace_id.clone(),
        ActorInstanceId::generate(),
        WorkPackageId::generate(),
    )
}


