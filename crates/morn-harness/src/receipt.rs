//! Execution receipts: auditable proof that an external execution happened.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    ExecutionReceiptId, Id, PrincipalId, RuntimeBindingId, WorkPackageId, WorkspaceId,
};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::context::RuntimeContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExecutorOutcomeReconciliationTag;
pub type ExecutorOutcomeReconciliationId = Id<ExecutorOutcomeReconciliationTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorOutcomeDisposition {
    NoEffectConfirmed,
    EffectObserved,
    StillUnknown,
}

/// Deployment-issued one-shot authorization to resolve one ambiguous executor
/// receipt. HTTP/UI callers may present this bearer reference but cannot mint
/// or widen its Work/generation/binding/receipt scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorOutcomeReconciliationAuthorization {
    pub authorization_id: String,
    pub principal_id: PrincipalId,
    pub work_package_id: WorkPackageId,
    pub work_generation: u64,
    pub execution_binding_ref: RuntimeBindingId,
    pub execution_receipt_id: ExecutionReceiptId,
    pub disposition: ExecutorOutcomeDisposition,
    pub evidence_refs: Vec<String>,
    pub issued_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl ExecutorOutcomeReconciliationAuthorization {
    pub fn validate(&self) -> morn_kernel::error::Result<()> {
        if self.authorization_id.trim().is_empty()
            || self.work_generation == 0
            || self.evidence_refs.is_empty()
            || self
                .evidence_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(morn_kernel::error::Error::validation(
                "executor reconciliation authorization requires id, generation and deployment evidence",
            ));
        }
        if self.valid_until.is_some_and(|until| until < self.issued_at) {
            return Err(morn_kernel::error::Error::validation(
                "executor reconciliation authorization validity cannot end before issue time",
            ));
        }
        Ok(())
    }

    pub fn authorizes(
        &self,
        work_package_id: &WorkPackageId,
        work_generation: u64,
        execution_binding_ref: &RuntimeBindingId,
        execution_receipt_id: &ExecutionReceiptId,
        now: Timestamp,
    ) -> bool {
        self.validate().is_ok()
            && &self.work_package_id == work_package_id
            && self.work_generation == work_generation
            && &self.execution_binding_ref == execution_binding_ref
            && &self.execution_receipt_id == execution_receipt_id
            && self.issued_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
}

/// Durable record produced only after an exact deployment authorization is
/// consumed. This is control evidence about executor settlement, not a business
/// ObservedOutcome and not Acceptance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorOutcomeReconciliation {
    pub id: ExecutorOutcomeReconciliationId,
    pub authorization_id: String,
    pub principal_id: PrincipalId,
    pub work_package_id: WorkPackageId,
    pub work_generation: u64,
    pub execution_binding_ref: RuntimeBindingId,
    pub execution_receipt_id: ExecutionReceiptId,
    pub disposition: ExecutorOutcomeDisposition,
    pub evidence_refs: Vec<String>,
    pub reason: String,
    pub reconciled_at: Timestamp,
}

impl ExecutorOutcomeReconciliation {
    pub fn from_authorization(
        authorization: &ExecutorOutcomeReconciliationAuthorization,
        reason: impl Into<String>,
        reconciled_at: Timestamp,
    ) -> morn_kernel::error::Result<Self> {
        authorization.validate()?;
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(morn_kernel::error::Error::validation(
                "executor outcome reconciliation requires a reason",
            ));
        }
        Ok(Self {
            id: ExecutorOutcomeReconciliationId::generate_with("exec-reconcile"),
            authorization_id: authorization.authorization_id.clone(),
            principal_id: authorization.principal_id.clone(),
            work_package_id: authorization.work_package_id.clone(),
            work_generation: authorization.work_generation,
            execution_binding_ref: authorization.execution_binding_ref.clone(),
            execution_receipt_id: authorization.execution_receipt_id.clone(),
            disposition: authorization.disposition,
            evidence_refs: authorization.evidence_refs.clone(),
            reason,
            reconciled_at,
        })
    }
}

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
    fn executor_reconciliation_authorization_is_exact_time_bounded_and_evidence_backed() {
        let work = WorkPackageId::generate_with("work");
        let binding = RuntimeBindingId::generate_with("binding");
        let receipt = ExecutionReceiptId::generate_with("receipt");
        let authorization = ExecutorOutcomeReconciliationAuthorization {
            authorization_id: "exec-auth-1".to_string(),
            principal_id: PrincipalId::generate_with("operator"),
            work_package_id: work.clone(),
            work_generation: 3,
            execution_binding_ref: binding.clone(),
            execution_receipt_id: receipt.clone(),
            disposition: ExecutorOutcomeDisposition::NoEffectConfirmed,
            evidence_refs: vec!["source://reconciliation/1".to_string()],
            issued_at: Timestamp::from_millis(10),
            valid_until: Some(Timestamp::from_millis(20)),
        };
        assert!(authorization.authorizes(&work, 3, &binding, &receipt, Timestamp::from_millis(15)));
        assert!(!authorization.authorizes(
            &work,
            4,
            &binding,
            &receipt,
            Timestamp::from_millis(15)
        ));
        assert!(!authorization.authorizes(
            &work,
            3,
            &binding,
            &receipt,
            Timestamp::from_millis(21)
        ));

        let record = ExecutorOutcomeReconciliation::from_authorization(
            &authorization,
            "authoritative audit confirmed no external effect",
            Timestamp::from_millis(15),
        )
        .unwrap();
        assert_eq!(
            record.disposition,
            ExecutorOutcomeDisposition::NoEffectConfirmed
        );
        assert_eq!(record.execution_receipt_id, receipt);
    }

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
