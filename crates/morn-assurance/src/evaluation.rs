//! Evaluation runner: executes steps under fault injection in isolated state.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{EvaluationResultId, EvaluationRunId, EvaluationSuiteId};
use morn_kernel::time::Timestamp;

use crate::simulation::{FaultInjection, FaultKind};

/// One step of the evaluation execution (mirrors a WorkNode/workflow step).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalStep {
    pub name: String,
    pub acceptance_required: bool,
    pub approval_required: bool,
    pub harness: String,
}

impl EvalStep {
    pub fn new(name: impl Into<String>, acceptance_required: bool) -> Self {
        Self {
            name: name.into(),
            acceptance_required,
            approval_required: false,
            harness: "morn-native".to_string(),
        }
    }
}

/// Decision produced by an evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EvaluationDecision {
    Pass,
    Fail,
    Conditional,
}

/// Structured evaluation result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub id: EvaluationResultId,
    pub scenario_id: String,
    pub solution_version: String,
    pub correctness: f64,
    pub acceptance: bool,
    pub policy: bool,
    pub provenance: bool,
    pub recovery: f64,
    pub outcome: bool,
    pub human_interventions: u32,
    pub latency_ms: u64,
    pub cost: f64,
    pub regressions: Vec<String>,
    pub failures: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub decision: EvaluationDecision,
    pub created_at: Timestamp,
}

impl EvaluationResult {
    pub fn decision_text(&self) -> &'static str {
        match self.decision {
            EvaluationDecision::Pass => "pass",
            EvaluationDecision::Fail => "fail",
            EvaluationDecision::Conditional => "conditional",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRun {
    pub id: EvaluationRunId,
    pub suite_id: EvaluationSuiteId,
    pub result: EvaluationResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationSuite {
    pub id: EvaluationSuiteId,
    pub name: String,
    pub scenarios: Vec<String>,
}

/// Isolated evaluation runner: reads a snapshot, writes no production state.
#[derive(Debug, Default)]
pub struct EvaluationRunner {
    pub runs: Vec<EvaluationRun>,
}

impl EvaluationRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run a set of steps with injected faults. `previous_failures` are used
    /// for regression comparison against a previous solution version.
    pub fn run(
        &mut self,
        scenario_id: &str,
        solution_version: &str,
        steps: &[EvalStep],
        faults: &[FaultInjection],
        previous_failures: &[String],
    ) -> EvaluationResult {
        let mut failures = Vec::new();
        let mut regressions = Vec::new();
        let mut completed = 0usize;
        let mut human_interventions = 0u32;
        let mut policy_ok = true;
        let mut provenance_ok = true;
        let mut recovered = 0f64;
        let total = steps.len().max(1) as f64;
        let mut cost = 0.0f64;

        for step in steps {
            let fault = faults
                .iter()
                .find(|f| f.target_step == step.name)
                .map(|f| f.kind);
            match fault {
                None => {
                    completed += 1;
                    cost += 1.0;
                    if step.approval_required {
                        human_interventions += 1;
                    }
                }
                Some(FaultKind::ToolTimeout | FaultKind::ToolError | FaultKind::MalformedData) => {
                    failures.push(format!("{}: tool failure", step.name));
                    recovered += 0.5; // retry recovers half the time
                }
                Some(FaultKind::HarnessCrash | FaultKind::ModelUnavailable) => {
                    failures.push(format!("{}: harness/model unavailable", step.name));
                    recovered += 0.0;
                }
                Some(FaultKind::PermissionDenied) => {
                    failures.push(format!("{}: permission denied", step.name));
                    policy_ok = false;
                }
                Some(FaultKind::ApprovalMissing) => {
                    failures.push(format!("{}: approval missing (E3 gate)", step.name));
                    human_interventions += 1;
                    policy_ok = false;
                }
                Some(FaultKind::UntrustedContext) => {
                    failures.push(format!("{}: untrusted context", step.name));
                    provenance_ok = false;
                }
                Some(FaultKind::RepresentationViolation) => {
                    failures.push(format!("{}: representation boundary violation", step.name));
                    policy_ok = false;
                }
                Some(FaultKind::EvidenceConflict) => {
                    failures.push(format!("{}: evidence conflict", step.name));
                    completed += 1;
                }
                Some(FaultKind::BudgetExhausted) => {
                    failures.push("budget exhausted".to_string());
                    break;
                }
                Some(FaultKind::DeadlineExpired) => {
                    failures.push("deadline expired".to_string());
                    break;
                }
            }
        }

        let correctness = completed as f64 / total;
        let acceptance = completed == steps.len();
        let recovery = recovered;
        let outcome = acceptance && policy_ok;

        for f in &failures {
            if previous_failures.contains(f) {
                regressions.push(format!("regression vs previous: {f}"));
            }
        }

        let decision = if !policy_ok {
            EvaluationDecision::Fail
        } else if failures.is_empty() {
            EvaluationDecision::Pass
        } else {
            EvaluationDecision::Conditional
        };

        let result = EvaluationResult {
            id: EvaluationResultId::generate_with("evres"),
            scenario_id: scenario_id.to_string(),
            solution_version: solution_version.to_string(),
            correctness,
            acceptance,
            policy: policy_ok,
            provenance: provenance_ok,
            recovery,
            outcome,
            human_interventions,
            latency_ms: (completed as u64) * 100,
            cost,
            regressions,
            failures,
            evidence_refs: vec![
                "execution_receipt".to_string(),
                "artifact_version".to_string(),
            ],
            decision,
            created_at: Timestamp::now(),
        };
        self.runs.push(EvaluationRun {
            id: EvaluationRunId::generate_with("evrun"),
            suite_id: EvaluationSuiteId::generate_with("evsuite"),
            result: result.clone(),
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{FaultInjection, FaultKind};

    fn steps() -> Vec<EvalStep> {
        vec![
            EvalStep::new("collect", true),
            EvalStep::new("analyze", true),
            EvalStep::new("review", true),
            EvalStep::new("approve", false),
            EvalStep::new("release", true),
        ]
    }

    #[test]
    fn clean_run_passes() {
        let mut runner = EvaluationRunner::new();
        let result = runner.run("s1", "sol-1.0", &steps(), &[], &[]);
        assert_eq!(result.decision, EvaluationDecision::Pass);
        assert!(result.acceptance);
        assert!(result.policy);
        assert_eq!(result.correctness, 1.0);
    }

    #[test]
    fn approval_missing_fails_policy() {
        let mut runner = EvaluationRunner::new();
        let faults = vec![FaultInjection::new(
            FaultKind::ApprovalMissing,
            "release",
            "no pi approval",
        )];
        let result = runner.run("s2", "sol-1.0", &steps(), &faults, &[]);
        assert_eq!(result.decision, EvaluationDecision::Fail);
        assert!(!result.policy);
        assert_eq!(result.human_interventions, 1);
    }

    #[test]
    fn tool_failure_is_conditional_with_recovery() {
        let mut runner = EvaluationRunner::new();
        let faults = vec![FaultInjection::new(
            FaultKind::ToolError,
            "analyze",
            "pipeline exit 1",
        )];
        let result = runner.run("s3", "sol-1.0", &steps(), &faults, &[]);
        assert_eq!(result.decision, EvaluationDecision::Conditional);
        assert!(result.recovery > 0.0);
        assert!(!result.acceptance);
    }

    #[test]
    fn regression_detected_against_previous() {
        let mut runner = EvaluationRunner::new();
        let faults = vec![FaultInjection::new(
            FaultKind::PermissionDenied,
            "review",
            "denied",
        )];
        let previous = vec!["review: permission denied".to_string()];
        let result = runner.run("s4", "sol-1.1", &steps(), &faults, &previous);
        assert_eq!(result.decision, EvaluationDecision::Fail);
        assert!(result
            .regressions
            .iter()
            .any(|r| r.contains("review: permission denied")));
    }

    #[test]
    fn budget_exhausted_stops_run() {
        let mut runner = EvaluationRunner::new();
        let faults = vec![FaultInjection::new(
            FaultKind::BudgetExhausted,
            "analyze",
            "over budget",
        )];
        let result = runner.run("s5", "sol-1.0", &steps(), &faults, &[]);
        assert!(result.failures.iter().any(|f| f == "budget exhausted"));
        assert_eq!(result.decision, EvaluationDecision::Conditional);
    }
}
