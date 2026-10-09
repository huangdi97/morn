//! Execution receipts: auditable proof that an external execution happened.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ExecutionReceiptId, RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::context::RuntimeContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub id: ExecutionReceiptId,
    pub workspace_id: WorkspaceId,
    #[serde(default)]
    pub work_package_id: Option<WorkPackageId>,
    #[serde(default)]
    pub work_generation: Option<u64>,
    #[serde(default)]
    pub execution_binding_ref: Option<RuntimeBindingId>,
    #[serde(default)]
    pub provider_ref: Option<String>,
    #[serde(default)]
    pub scope_ref: Option<String>,
    #[serde(default)]
    pub execution_environment_ref: Option<String>,
    pub session_id: String,
    pub trace_refs: Vec<String>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub outcome: String,
    pub harness_version: Option<Version>,
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub runtime_digest: Option<String>,
    pub event_ids: Vec<String>,
}

impl ExecutionReceipt {
    pub fn new(workspace_id: WorkspaceId, session_id: impl Into<String>) -> Self {
        Self {
            id: ExecutionReceiptId::generate_with("rcpt"),
            workspace_id,
            work_package_id: None,
            work_generation: None,
            execution_binding_ref: None,
            provider_ref: None,
            scope_ref: None,
            execution_environment_ref: None,
            session_id: session_id.into(),
            trace_refs: Vec::new(),
            started_at: Timestamp::now(),
            ended_at: None,
            outcome: "running".to_string(),
            harness_version: None,
            runtime_version: None,
            runtime_digest: None,
            event_ids: Vec::new(),
        }
    }

    pub fn from_runtime_context(
        ctx: &RuntimeContext,
        provider_ref: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        let mut receipt = Self::new(ctx.workspace_id.clone(), session_id);
        receipt.work_package_id = Some(ctx.work_package_id.clone());
        receipt.work_generation = ctx.work_generation;
        receipt.execution_binding_ref = ctx.execution_binding_ref.clone();
        receipt.provider_ref = Some(provider_ref.into());
        receipt.scope_ref = ctx.scope_id.clone();
        receipt.execution_environment_ref = ctx.execution_environment_ref.clone();
        receipt
    }

    pub fn is_pinned_work_evidence(&self) -> bool {
        self.work_package_id.is_some()
            && self
                .work_generation
                .is_some_and(|generation| generation > 0)
            && self.execution_binding_ref.is_some()
            && self
                .provider_ref
                .as_deref()
                .is_some_and(|provider| !provider.trim().is_empty())
            && self
                .scope_ref
                .as_deref()
                .is_some_and(|scope| !scope.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::ActorInstanceId;

    #[test]
    fn work_execution_receipt_requires_explicit_generation_binding_and_scope() {
        let base = RuntimeContext::new(
            WorkspaceId::generate(),
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        );
        assert!(
            !ExecutionReceipt::from_runtime_context(&base, "dsh", "s").is_pinned_work_evidence()
        );

        let pinned = base
            .with_work_binding(3, RuntimeBindingId::generate_with("binding"))
            .unwrap()
            .with_scope_id("scope://run")
            .unwrap();
        let receipt = ExecutionReceipt::from_runtime_context(&pinned, "dsh", "s");
        assert!(receipt.is_pinned_work_evidence());
        assert_eq!(receipt.work_generation, Some(3));
    }
}
