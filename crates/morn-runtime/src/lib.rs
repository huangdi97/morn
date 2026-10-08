//! Morn execution runtime.
//!
//! The legacy ActionGateway/RuntimeProvider remain supported. v11.5 adds
//! execution bindings, explicit attempt lifecycle, reconciliation and a
//! provider-neutral execution-environment contract so provider sessions and
//! sandboxes can never become canonical business truth.

pub mod attempt;
pub mod authority;
pub mod binding;
pub mod composition;
pub mod credentials;
pub mod durable_workflow;
pub mod environment;
pub mod execution_manifest;
pub mod gateway;
pub mod provider_registry;
pub mod reconciliation;
pub mod runtime_provider;
pub mod workload_identity;

pub use attempt::{ActionAttempt, ActionAttemptId, AttemptState, CancellationDisposition};
pub use authority::{
    decide_bound, enforce_authority, AuthorityDecisionId, AuthorityDecisionRecord,
    AuthorityProvider, AuthorityRequest, BoundAuthorityDecision, NativePolicyAuthority,
};
pub use binding::{
    BindingMigrationDecision, BindingMigrationDecisionId, BindingMigrationReason, ExecutionBinding,
};
pub use composition::{
    run_composition_contract, CompositionProviderRef, CompositionRuntimeProvider,
    CompositionRuntimeSnapshot, CompositionSlotBinding, FixtureCompositionRuntime,
};
pub use credentials::{
    CredentialHandle, CredentialHandleId, CredentialProvider, CredentialRequest,
    FixtureCredentialProvider,
};
pub use durable_workflow::{
    DurableWorkflowBinding, DurableWorkflowBindingId, DurableWorkflowEvidence,
    DurableWorkflowProvider, DurableWorkflowState, LegacyMornDurableWorkflowProvider,
};
pub use environment::{
    ExecutionEnvironmentHandle, ExecutionEnvironmentId, ExecutionEnvironmentOffer,
    ExecutionEnvironmentProvider, ExecutionEnvironmentResolver, ExecutionEnvironmentSelection,
    ExecutionEnvironmentSpec, FixtureEnvironmentProvider, IsolationClass,
};
pub use execution_manifest::{CompositionRuntimeRef, ExecutionManifest};
pub use gateway::{
    ActionGateway, ActionPreview, AuthorizedAction, ExecutionOutcome, WorldCommitter,
};
pub use provider_registry::{
    reference_provider_catalog, ProviderDescriptor, ProviderFamily, ProviderObservation,
    ProviderRegistry, ProviderStatus,
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
