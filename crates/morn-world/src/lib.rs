//! Operational World L0: objects, relations, state, events, actions, goals, metrics, outcomes.
//! Canonical state may only change through `WorldService::commit_state` (governed commits).

pub mod action;
pub mod diff;
pub mod event;
pub mod goal;
pub mod object;
pub mod outcome;
pub mod relation;
pub mod service;
pub mod state;

pub use service::WorldService;
