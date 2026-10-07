//! Morn execution runtime.
//!
//! The legacy ActionGateway/RuntimeProvider remain supported. v11.5 adds
//! execution bindings, explicit attempt lifecycle, reconciliation and a
//! provider-neutral execution-environment contract so provider sessions and
//! sandboxes can never become canonical business truth.

pub mod attempt;
pub mod binding;
pub mod environment;
pub mod gateway;
pub mod reconciliation;
pub mod runtime_provider;

pub use attempt::{ActionAttempt, ActionAttemptId, AttemptState};
pub use binding::ExecutionBinding;
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
