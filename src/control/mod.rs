//! control — Morn Durable Control Plane (Phase 2).
//!
//! The durable work state is independent of any Cordis Fiber, Agent Session,
//! Sandbox Instance, or Workflow Execution. This module owns:
//! current projections, reconciliation, conditions, and outbox/lease runtime
//! support.
//!
//! INVARIANT: canonical progress never lives only in process memory — every
//! controller step reads from and writes to the store, is idempotent, and is
//! re-entrant after a crash.

pub mod controllers;
pub mod ddl;
pub mod projection;
pub mod store;
pub mod store_execution;
pub mod store_records;
pub mod store_runtime;
pub mod workcell;

pub use controllers::{
    AcceptanceController, AuthorityController, BindingController, CapabilityResolverController,
    ControllerOutcome, ExecutionController, OutcomeController, ProfileConformanceController,
    QualificationController, ReceiptController, ReconciliationController, SituationController,
    WorkController,
};
pub use projection::project_work;
pub use store::ControlPlaneStore;

/// Event schema version metadata stamped on every stored semantic event.
pub use workcell::{
    CandidateCapability, ExecutorKind, ResolutionFailure, ResolutionRequirements, Workcell,
    WorkcellMember, WorkcellResolver,
};
pub const EVENT_SCHEMA_VERSION: &str = "v1";
