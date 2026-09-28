//! solver — deterministic SolverProvider.
//!
//! A solver's output is a Candidate, never automatically a decision (master
//! §9.2). Provides at least one deterministic solver; verifies that a Workcell
//! does not require an Agent ("Agent is one executor").

use serde::{Deserialize, Serialize};

/// One scheduling/planning candidate produced by a solver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverCandidate {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Solver output: candidates + the constraints it checked. Candidates are NOT
/// decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverOutput {
    pub candidates: Vec<SolverCandidate>,
    /// Constraints the solver verified independently
    pub constraints_checked: Vec<String>,
}

/// A work-order assignment input for the solver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub work_ref: String,
    pub asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_order: Option<String>,
}

/// SolverProvider — deterministic optimization contract.
pub trait SolverProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn solve(&self, assignment: &Assignment) -> Result<SolverOutput, String>;
}

/// Reference CP-SAT-style deterministic solver: produces the four canonical
/// candidate plans (continue / immediate stop + reroute / split batch / no
/// action) for the equipment-exception scenario. Deterministic: the same
/// input always produces the same output, with no Agent and no LLM.
pub struct ReferenceCpsatSolver {
    provider_id: String,
}

impl ReferenceCpsatSolver {
    pub fn new() -> Self {
        Self {
            provider_id: "reference-cpsat".to_string(),
        }
    }
}

impl Default for ReferenceCpsatSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SolverProvider for ReferenceCpsatSolver {
    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn solve(&self, assignment: &Assignment) -> Result<SolverOutput, String> {
        if assignment.work_ref.is_empty() || assignment.asset.is_empty() {
            return Err("assignment requires work_ref and asset".to_string());
        }
        Ok(SolverOutput {
            candidates: vec![
                SolverCandidate {
                    id: format!("{}-A", assignment.work_ref),
                    label: "continue_2h_then_maintenance".to_string(),
                    detail: None,
                },
                SolverCandidate {
                    id: format!("{}-B", assignment.work_ref),
                    label: "immediate_stop_and_reroute".to_string(),
                    detail: None,
                },
                SolverCandidate {
                    id: format!("{}-C", assignment.work_ref),
                    label: "split_batch".to_string(),
                    detail: None,
                },
                SolverCandidate {
                    id: format!("{}-D", assignment.work_ref),
                    label: "no_action".to_string(),
                    detail: None,
                },
            ],
            constraints_checked: vec!["no_plc_write".to_string(), "no_quality_release".to_string()],
        })
    }
}
