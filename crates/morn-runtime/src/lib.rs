//! Morn execution runtime.
//!
//! The legacy ActionGateway/RuntimeProvider remain supported. v11.5 adds
//! execution bindings, explicit attempt lifecycle, reconciliation and a
//! provider-neutral execution-environment contract so provider sessions and
//! sandboxes can never become canonical business truth.

pub mod attempt;
pub mod authority;
pub mod binding;
pub mod credentials;
pub mod environment;
pub mod gateway;
pub mod reconciliation;
pub mod runtime_provider;
pub mod workload_identity;

pub use attempt::{ActionAttempt, ActionAttemptId, AttemptState};
pub use authority::{
    enforce_authority, AuthorityDecisionId, AuthorityDecisionRecord, AuthorityProvider,
    AuthorityRequest, NativePolicyAuthority,
};
pub use binding::{
    BindingMigrationDecision, BindingMigrationDecisionId, BindingMigrationReason, ExecutionBinding,
};
pub use credentials::{
    CredentialHandle, CredentialHandleId, CredentialProvider, CredentialRequest,
    FixtureCredentialProvider,
};
pub use environment::{
    ExecutionEnvironmentHandle, ExecutionEnvironmentId, ExecutionEnvironmentProvider,
    ExecutionEnvironmentSpec, FixtureEnvironmentProvider, IsolationClass,
};
pub use gateway::{
    ActionGateway, ActionPreview, AuthorizedAction, ExecutionOutcome, WorldCommitter,
};
pub use reconciliation::{
    reconcile_attempt, OutcomeReconciler, ReconciliationObservation, ReconciliationRecord,
    ReconciliationRecordId,
};
pub use runtime_provider::{run_runtime_conformance, FixtureRuntime, RuntimeProvider};

pub use workload_identity::{
    FixtureWorkloadIdentityProvider, WorkloadIdentity, WorkloadIdentityId,
    WorkloadIdentityProvider, WorkloadIdentityRequest,
};
