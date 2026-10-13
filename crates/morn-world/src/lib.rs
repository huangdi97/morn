//! Operational World L0: objects, relations, state, events, actions, goals, metrics, outcomes.
//! Canonical state may only change through `ActionGateway`; raw commit mutation is crate-private.

pub mod action;
pub mod diff;
pub mod event;
pub mod gateway;
pub mod goal;
pub mod object;
pub mod observed_outcome;
pub mod outcome;
pub mod relation;
pub mod service;
pub mod state;

pub use gateway::{
    ActionGateway, ActionPreview, ApprovalWitness, AuthorizedAction, ExecutionOutcome,
    WorldCommitter, WorldVerifier,
};
pub use observed_outcome::{ObservedOutcome, OutcomeSourceKind};
pub use service::WorldService;
