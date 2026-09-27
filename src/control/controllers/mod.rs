//! controllers — reference controllers that advance observed state toward
//! desired conditions.
//!
//! Each controller is a small deterministic worker: idempotent, re-entrant,
//! crash-safe. Canonical progress lives in the store (never in process
//! memory); controllers read from and write to the store, and output evidence
//! / condition changes. No single "super agent" holds all control.

pub mod execution_flow;
pub mod outcome_flow;
pub mod resolve_loop;
pub mod state_loop;

pub use execution_flow::{AuthorityController, BindingController, ExecutionController};
pub use outcome_flow::{
    AcceptanceController, OutcomeController, ReceiptController, ReconciliationController,
};
pub use resolve_loop::{CapabilityResolverController, QualificationController};
pub use state_loop::{ProfileConformanceController, SituationController, WorkController};

use crate::work::Condition;

/// What a controller changed in one reconcile pass.
#[derive(Debug, Clone, PartialEq)]
pub struct ControllerOutcome {
    pub controller: &'static str,
    pub work_ref: String,
    pub condition_changes: Vec<Condition>,
    pub notes: Vec<String>,
}

impl ControllerOutcome {
    pub fn new(controller: &'static str, work_ref: &str) -> Self {
        Self {
            controller,
            work_ref: work_ref.to_string(),
            condition_changes: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition_changes.push(condition);
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}
