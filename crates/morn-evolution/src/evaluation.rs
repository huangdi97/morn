//! Evaluation of an evolution branch.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{EvolutionBranchId, EvolutionEvaluationId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionEvaluation {
    pub id: EvolutionEvaluationId,
    pub branch_id: EvolutionBranchId,
    pub correctness: bool,
    pub regression: bool,
    pub policy_ok: bool,
    pub cost_latency: Option<bool>,
    pub acceptance_outcome: Option<bool>,
    pub pass: bool,
    pub evidence: String,
    pub created_at: Timestamp,
}

impl EvolutionEvaluation {
    pub fn new(
        branch_id: EvolutionBranchId,
        correctness: bool,
        regression: bool,
        policy_ok: bool,
        evidence: impl Into<String>,
    ) -> Self {
        let pass = correctness && regression && policy_ok;
        Self {
            id: EvolutionEvaluationId::generate_with("eve"),
            branch_id,
            correctness,
            regression,
            policy_ok,
            cost_latency: None,
            acceptance_outcome: None,
            pass,
            evidence: evidence.into(),
            created_at: Timestamp::now(),
        }
    }

    pub fn passed(&self) -> bool {
        self.pass
    }
}
