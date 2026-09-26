//! work — Morn Semantic Specification (language-neutral contracts, Rust mapping).
//!
//! This module defines the protocol resources of the v11.5 master as serde
//! types mirroring the JSON Schemas in `schemas/`. It is persistence-free and
//! provider-free by design: no storage engine, no Cordis/DSH/Pi types. The
//! durable control plane (Phase 2) stores these; controllers and providers
//! (Phase 2+) produce and consume them.
//!
//! Top-level principles observable here:
//! Work is the unit of coordination; Capability is the unit of composition;
//! Accepted Outcome is the unit of value; Agent is one kind of executor.

pub mod authority;
pub mod capability;
pub mod common;
pub mod condition;
pub mod execution;
pub mod outcome;
pub mod principal;
pub mod profile;
pub mod resource;
pub mod validation;
pub mod world;

pub use authority::{
    AuthorityDecision, AuthorityDecisionType, AuthorityEvidence, AuthorityRequest, DelegationDepth,
    TimeWindow,
};
pub use capability::{
    AuthorityCeiling, CapabilityLifecycle, CapabilityManifest, CapabilityRelease, CapabilitySpec,
    CapabilityState, Economics, ExecutionRequirements, QualificationRecord, QualificationStatus,
    SideEffects, SiteAdmission,
};
pub use common::{HistoricalRecord, ObjectRef, Provenance, SourceRef};
pub use condition::{Condition, ConditionStatus, ConditionType};
pub use execution::{
    ActionAttempt, AttemptStatus, BindingStatus, ExecutionBinding, Receipt, ReconciliationFound,
    ReconciliationRecord, ReconciliationResult,
};
pub use outcome::{
    AcceptanceDecision, AcceptanceType, ObservedOutcome, OutcomeMetric, OutcomeSource,
    ValueAssessment,
};
pub use principal::{Actor, Commitment, Principal, Role};
pub use profile::{Profile, ProfileRequirement, RequirementLevel, SolutionPackage};
pub use resource::{
    now_iso, AcceptanceSpec, AuthorityRequirement, Budget, CostAmount, Work, WorkAcceptanceState,
    WorkPhase, WorkSpec, WorkStatus,
};
pub use validation::Schema;
pub use world::{Observation, Situation, SituationStatus};

#[cfg(test)]
mod tests;

/// Version of the semantic contract implemented by this module.
pub const SEMANTIC_CONTRACT_VERSION: &str = "v1";
