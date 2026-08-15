//! Fault injection and simulation scenarios.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    FaultInjectionId, SimulationRunId, SimulationScenarioId, SolutionVersionId,
};
use morn_kernel::time::Timestamp;

/// Kinds of injectable faults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum FaultKind {
    ToolTimeout,
    ToolError,
    HarnessCrash,
    ModelUnavailable,
    PermissionDenied,
    ApprovalMissing,
    UntrustedContext,
    RepresentationViolation,
    EvidenceConflict,
    MalformedData,
    BudgetExhausted,
    DeadlineExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultInjection {
    pub id: FaultInjectionId,
    pub kind: FaultKind,
    pub target_step: String,
    pub detail: String,
}

impl FaultInjection {
    pub fn new(kind: FaultKind, target_step: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: FaultInjectionId::generate_with("fault"),
            kind,
            target_step: target_step.into(),
            detail: detail.into(),
        }
    }
}

/// A simulation scenario: snapshot + inputs + faults + expected invariants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationScenario {
    pub id: SimulationScenarioId,
    pub scenario_type: String,
    pub base_world_snapshot: serde_json::Value,
    pub solution_version: Option<SolutionVersionId>,
    pub inputs: serde_json::Value,
    pub fault_injections: Vec<FaultInjection>,
    pub mocked_or_sandboxed_actions: bool,
    pub expected_invariants: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub created_at: Timestamp,
}

impl SimulationScenario {
    pub fn new(
        scenario_type: impl Into<String>,
        base_world_snapshot: serde_json::Value,
        inputs: serde_json::Value,
    ) -> Self {
        Self {
            id: SimulationScenarioId::generate_with("scen"),
            scenario_type: scenario_type.into(),
            base_world_snapshot,
            solution_version: None,
            inputs,
            fault_injections: Vec::new(),
            mocked_or_sandboxed_actions: true,
            expected_invariants: Vec::new(),
            stop_conditions: Vec::new(),
            created_at: Timestamp::now(),
        }
    }

    pub fn add_fault(mut self, fault: FaultInjection) -> Self {
        self.fault_injections.push(fault);
        self
    }
}

/// A finished simulation run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationRun {
    pub id: SimulationRunId,
    pub scenario_id: SimulationScenarioId,
    pub completed: bool,
    pub stopped_by: Option<String>,
    pub started_at: Timestamp,
}
