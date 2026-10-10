//! Shared provider contract suite. Every harness provider path must pass it,
//! proving Morn-side invariants independent of the concrete provider.

use morn_kernel::error::Result;
use morn_kernel::ids::WorkspaceId;

use crate::context::RuntimeContext;
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
/// Covers the baseline lifecycle plus explicitly advertised optional features,
/// E0 unmount cleanup, scope isolation, and event normalization to Morn
/// `ExecutionEvent`s. Unsupported lifecycle operations are not fabricated.
pub fn run_provider_contract(
    provider: &mut dyn HarnessProvider,
    ctx: &RuntimeContext,
) -> Result<ContractReport> {
    let mut report: Vec<(String, bool)> = Vec::new();
    let provider_name = provider.provider_name().to_string();

    // lifecycle
    let mut contract_scope = CapabilityScope::new(
        ScopeKind::Workcell,
        None,
        ctx.workspace_id.clone(),
        "contract-scope",
    );
    for restriction in provider.required_scope_restrictions() {
        contract_scope = contract_scope.with_restriction(*restriction);
    }
    let mount_result = provider.mount(contract_scope);
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

    let scoped_ctx = match ctx.clone().with_scope_id(handle.scope_id.clone()) {
        Ok(context) => context,
        Err(error) => {
            let _ = provider.unmount(&handle);
            return Err(morn_kernel::error::Error::validation(error));
        }
    };
    let session = match provider.start(&scoped_ctx) {
        Ok(s) => {
            check(&mut report, "start", true);
            s
        }
        Err(e) => {
            check(&mut report, "start", false);
            let _ = provider.unmount(&handle);
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
            && events.iter().all(|event| {
                event.workspace_id == scoped_ctx.workspace_id
                    && event.session_id == session.id
                    && !event.summary.trim().is_empty()
            }),
    );

    check(
        &mut report,
        "inspect",
        provider.inspect(&session.id).is_ok_and(|snapshot| {
            snapshot.session_id == session.id && !snapshot.status.trim().is_empty()
        }),
    );

    let features = provider.features();
    if features.interrupt {
        check(
            &mut report,
            "interrupt",
            provider.interrupt(&session.id).is_ok(),
        );
    } else {
        check(&mut report, "interrupt unsupported explicitly", true);
    }

    if features.resume {
        if features.interrupt {
            check(&mut report, "resume", provider.resume(&session.id).is_ok());
        } else {
            check(&mut report, "resume requires interrupt capability", false);
        }
    } else {
        check(&mut report, "resume unsupported explicitly", true);
    }

    if features.session_close {
        let receipt = provider.terminate(&session.id);
        check(
            &mut report,
            "terminate -> receipt",
            receipt.is_ok_and(|r| !r.trace_refs.is_empty()),
        );
    } else {
        check(&mut report, "session close unsupported explicitly", true);
    }

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
