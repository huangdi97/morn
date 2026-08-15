//! Shadow: run baseline and candidate on the same inputs with isolated side
//! effects, compare, and produce a readiness decision.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ShadowComparisonId, ShadowProfileId, ShadowRunId};
use morn_kernel::time::Timestamp;

use crate::evaluation::{EvaluationDecision, EvaluationResult};

/// A shadow profile: baseline vs candidate for the same inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowProfile {
    pub id: ShadowProfileId,
    pub name: String,
    pub baseline_solution: String,
    pub candidate_solution: String,
    pub inputs: Value,
}

impl ShadowProfile {
    pub fn new(
        name: impl Into<String>,
        baseline_solution: impl Into<String>,
        candidate_solution: impl Into<String>,
        inputs: Value,
    ) -> Self {
        Self {
            id: ShadowProfileId::generate_with("shadowp"),
            name: name.into(),
            baseline_solution: baseline_solution.into(),
            candidate_solution: candidate_solution.into(),
            inputs,
        }
    }
}

/// Readiness decision for the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Readiness {
    Ready,
    NotReady,
    Conditional,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowComparison {
    pub id: ShadowComparisonId,
    pub baseline_eval: EvaluationResult,
    pub candidate_eval: EvaluationResult,
    pub readiness: Readiness,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowRun {
    pub id: ShadowRunId,
    pub profile_id: ShadowProfileId,
    pub comparison: ShadowComparison,
    pub isolated_side_effects: bool,
    pub created_at: Timestamp,
}

/// Runs baseline and candidate evaluations in isolation and compares.
#[derive(Debug, Default)]
pub struct ShadowRunner {
    pub runs: Vec<ShadowRun>,
}

impl ShadowRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compare two already-produced evaluation results (baseline vs candidate).
    /// The runner records the comparison; it never executes external actions.
    pub fn compare(
        &mut self,
        profile_id: ShadowProfileId,
        baseline_eval: EvaluationResult,
        candidate_eval: EvaluationResult,
    ) -> ShadowRun {
        let mut notes = Vec::new();
        if candidate_eval.cost > baseline_eval.cost {
            notes.push(format!(
                "candidate cost higher: {} vs {}",
                candidate_eval.cost, baseline_eval.cost
            ));
        }
        if candidate_eval.correctness < baseline_eval.correctness {
            notes.push("candidate correctness lower than baseline".to_string());
        }
        if !candidate_eval.policy && baseline_eval.policy {
            notes.push("candidate introduces a policy violation".to_string());
        }

        let readiness = match (baseline_eval.decision, candidate_eval.decision) {
            (_, EvaluationDecision::Fail) => Readiness::NotReady,
            (EvaluationDecision::Pass, EvaluationDecision::Pass) => Readiness::Ready,
            (EvaluationDecision::Pass, EvaluationDecision::Conditional) => Readiness::Conditional,
            (EvaluationDecision::Conditional, EvaluationDecision::Conditional) => {
                Readiness::Conditional
            }
            (_, EvaluationDecision::Pass) => Readiness::Ready,
            _ => Readiness::Conditional,
        };

        let run = ShadowRun {
            id: ShadowRunId::generate_with("shadowrun"),
            profile_id,
            comparison: ShadowComparison {
                id: ShadowComparisonId::generate_with("shadowcmp"),
                baseline_eval,
                candidate_eval,
                readiness,
                notes,
            },
            isolated_side_effects: true,
            created_at: Timestamp::now(),
        };
        self.runs.push(run.clone());
        run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::EvaluationResult;
    use crate::evaluation::{EvalStep, EvaluationRunner};
    use crate::simulation::{FaultInjection, FaultKind};

    fn eval(
        runner: &mut EvaluationRunner,
        tag: &str,
        faults: &[FaultInjection],
    ) -> EvaluationResult {
        let steps = vec![
            EvalStep::new("collect", true),
            EvalStep::new("analyze", true),
            EvalStep::new("review", true),
            EvalStep::new("release", true),
        ];
        runner.run(tag, "sol", &steps, faults, &[])
    }

    #[test]
    fn candidate_worse_fails_readiness() {
        let mut runner = EvaluationRunner::new();
        let baseline = eval(&mut runner, "base", &[]);
        let candidate = eval(
            &mut runner,
            "cand",
            &[FaultInjection::new(
                FaultKind::PermissionDenied,
                "release",
                "denied",
            )],
        );
        let mut shadow = ShadowRunner::new();
        let run = shadow.compare(ShadowProfileId::generate(), baseline, candidate);
        assert_eq!(run.comparison.readiness, Readiness::NotReady);
        assert!(run.isolated_side_effects);
    }

    #[test]
    fn equal_pass_is_ready() {
        let mut runner = EvaluationRunner::new();
        let baseline = eval(&mut runner, "base", &[]);
        let candidate = eval(&mut runner, "cand", &[]);
        let mut shadow = ShadowRunner::new();
        let run = shadow.compare(ShadowProfileId::generate(), baseline, candidate);
        assert_eq!(run.comparison.readiness, Readiness::Ready);
    }

    #[test]
    fn candidate_conditional_is_conditional() {
        let mut runner = EvaluationRunner::new();
        let baseline = eval(&mut runner, "base", &[]);
        let candidate = eval(
            &mut runner,
            "cand",
            &[FaultInjection::new(
                FaultKind::ToolError,
                "analyze",
                "flaky",
            )],
        );
        let mut shadow = ShadowRunner::new();
        let run = shadow.compare(ShadowProfileId::generate(), baseline, candidate);
        assert_eq!(run.comparison.readiness, Readiness::Conditional);
    }
}
