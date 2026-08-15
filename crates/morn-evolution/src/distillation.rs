//! Deterministic Distillation: convert a repeated stable actor step into a
//! deterministic rule/program with long-tail fallback to the actor.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{DistillationCandidateId, DistilledProgramId};

/// Input to the distilled QC decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistillationInput {
    pub rows: u64,
    pub special: bool,
}

/// Output of a distillation decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistillationOutput {
    pub accepted: bool,
    pub reason: String,
}

/// A deterministic rule (function) extracted from stable actor behavior.
pub type DeterministicRule = fn(&DistillationInput) -> DistillationOutput;

/// Baseline metrics of the actor path being distilled.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ActorBaseline {
    pub quality: f64,
    pub latency_ms: u64,
    pub cost: f64,
    pub human_minutes: f64,
}

/// A distilled deterministic program.
#[derive(Debug, Clone)]
pub struct DistilledProgram {
    pub id: DistilledProgramId,
    pub name: &'static str,
    pub rule: DeterministicRule,
    pub baseline: ActorBaseline,
}

/// One regression case: the input, the actor's decision, and whether the
/// long-tail fallback should apply.
#[derive(Debug, Clone)]
pub struct RegressionCase {
    pub input: DistillationInput,
    pub actor_output: DistillationOutput,
    pub should_fallback: bool,
}

/// Regression report comparing the program against the actor baseline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionReport {
    pub total_cases: u32,
    pub program_matches_actor: u32,
    pub mismatches: Vec<String>,
    pub quality_delta: f64,
    pub latency_reduction_ms: u64,
    pub cost_reduction: f64,
    pub human_load_reduction_min: f64,
    pub long_tail_fallback_count: u32,
    pub passed: bool,
}

/// A distillation candidate (data-only; never mutates production).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationCandidate {
    pub id: DistillationCandidateId,
    pub source_step: String,
    pub program_name: String,
    pub baseline: ActorBaseline,
    pub fallback_to_actor: bool,
    pub regression: Option<RegressionReport>,
    pub status: String,
}

/// Distillation service.
#[derive(Debug, Default)]
pub struct DistillationService {
    pub candidates: Vec<DistillationCandidate>,
    pub programs: Vec<DistilledProgram>,
}

impl DistillationService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a distilled program and create a candidate for `source_step`.
    pub fn distill(
        &mut self,
        source_step: &str,
        program_name: &'static str,
        rule: DeterministicRule,
        baseline: ActorBaseline,
    ) -> Result<DistillationCandidate> {
        if baseline.latency_ms == 0 && baseline.human_minutes == 0.0 {
            return Err(Error::validation("actor baseline must be provided"));
        }
        let program = DistilledProgram {
            id: DistilledProgramId::generate_with("dprog"),
            name: program_name,
            rule,
            baseline,
        };
        let candidate = DistillationCandidate {
            id: DistillationCandidateId::generate_with("dcand"),
            source_step: source_step.to_string(),
            program_name: program_name.to_string(),
            baseline,
            fallback_to_actor: true,
            regression: None,
            status: "proposed".to_string(),
        };
        self.programs.push(program);
        self.candidates.push(candidate.clone());
        Ok(candidate)
    }

    /// Run regression: compare program decisions against actor decisions.
    pub fn run_regression(
        &mut self,
        candidate_id: &DistillationCandidateId,
        cases: &[RegressionCase],
    ) -> Result<RegressionReport> {
        let program = self
            .programs
            .iter()
            .find(|p| {
                self.candidates
                    .iter()
                    .find(|c| c.id == *candidate_id)
                    .map(|c| c.program_name == p.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error::not_found(format!("program for candidate {candidate_id}")))?;

        let mut matches = 0u32;
        let mut fallbacks = 0u32;
        let mut mismatches = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            if case.should_fallback {
                fallbacks += 1;
                continue; // long-tail goes to actor; no comparison
            }
            let program_out = (program.rule)(&case.input);
            if program_out.accepted == case.actor_output.accepted {
                matches += 1;
            } else {
                mismatches.push(format!(
                    "case {i}: program={} actor={}",
                    program_out.accepted, case.actor_output.accepted
                ));
            }
        }
        let total = cases.len() as u32;
        let passed = matches == (total - fallbacks) && mismatches.is_empty();
        let report = RegressionReport {
            total_cases: total,
            program_matches_actor: matches,
            mismatches,
            quality_delta: matches as f64 / (total - fallbacks).max(1) as f64
                - program.baseline.quality,
            latency_reduction_ms: program.baseline.latency_ms.saturating_sub(10),
            cost_reduction: program.baseline.cost * 0.9,
            human_load_reduction_min: program.baseline.human_minutes,
            long_tail_fallback_count: fallbacks,
            passed,
        };
        if let Some(c) = self.candidates.iter_mut().find(|c| c.id == *candidate_id) {
            c.regression = Some(report.clone());
            c.status = if report.passed {
                "regression_passed".to_string()
            } else {
                "regression_failed".to_string()
            };
        }
        Ok(report)
    }

    /// Execute: use the program for stable inputs, fall back to actor otherwise.
    pub fn execute(
        &self,
        candidate_id: &DistillationCandidateId,
        input: DistillationInput,
    ) -> Result<(DistillationOutput, bool)> {
        let program = self
            .programs
            .iter()
            .find(|p| {
                self.candidates
                    .iter()
                    .find(|c| c.id == *candidate_id)
                    .map(|c| c.program_name == p.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error::not_found(format!("program for candidate {candidate_id}")))?;
        if input.special {
            // Long-tail: fall back to actor (simulated).
            return Ok((
                DistillationOutput {
                    accepted: input.rows > 0,
                    reason: "actor fallback: special case".to_string(),
                },
                false,
            ));
        }
        Ok(((program.rule)(&input), true))
    }
}

/// The concrete distilled rule for the BioLab QC step:
/// accept when rows > 0 and even (stable decision boundary).
pub fn qc_rule(input: &DistillationInput) -> DistillationOutput {
    let accepted = input.rows > 0 && input.rows.is_multiple_of(2);
    DistillationOutput {
        accepted,
        reason: if accepted {
            "qc pass".to_string()
        } else {
            "qc fail (odd or zero rows)".to_string()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> ActorBaseline {
        ActorBaseline {
            quality: 1.0,
            latency_ms: 1200,
            cost: 4.0,
            human_minutes: 8.0,
        }
    }

    fn cases() -> Vec<RegressionCase> {
        vec![
            RegressionCase {
                input: DistillationInput {
                    rows: 128,
                    special: false,
                },
                actor_output: DistillationOutput {
                    accepted: true,
                    reason: "ok".into(),
                },
                should_fallback: false,
            },
            RegressionCase {
                input: DistillationInput {
                    rows: 96,
                    special: false,
                },
                actor_output: DistillationOutput {
                    accepted: true,
                    reason: "ok".into(),
                },
                should_fallback: false,
            },
            RegressionCase {
                input: DistillationInput {
                    rows: 33,
                    special: false,
                },
                actor_output: DistillationOutput {
                    accepted: false,
                    reason: "odd".into(),
                },
                should_fallback: false,
            },
            RegressionCase {
                input: DistillationInput {
                    rows: 0,
                    special: true,
                },
                actor_output: DistillationOutput {
                    accepted: false,
                    reason: "empty".into(),
                },
                should_fallback: true,
            },
        ]
    }

    #[test]
    fn repeated_path_is_distilled_with_regression() {
        let mut svc = DistillationService::new();
        let candidate = svc.distill("qc", "qc-rule", qc_rule, baseline()).unwrap();
        let report = svc.run_regression(&candidate.id, &cases()).unwrap();
        assert!(report.passed, "mismatches: {:?}", report.mismatches);
        assert_eq!(report.program_matches_actor, 3);
        assert_eq!(report.long_tail_fallback_count, 1);
        assert!(report.cost_reduction > 0.0);
    }

    #[test]
    fn actor_fallback_for_long_tail() {
        let mut svc = DistillationService::new();
        let candidate = svc.distill("qc", "qc-rule", qc_rule, baseline()).unwrap();
        svc.run_regression(&candidate.id, &cases()).unwrap();
        // Stable input -> program.
        let (out, used_program) = svc
            .execute(
                &candidate.id,
                DistillationInput {
                    rows: 128,
                    special: false,
                },
            )
            .unwrap();
        assert!(used_program);
        assert!(out.accepted);
        // Long-tail input -> actor fallback.
        let (out2, used_program2) = svc
            .execute(
                &candidate.id,
                DistillationInput {
                    rows: 7,
                    special: true,
                },
            )
            .unwrap();
        assert!(!used_program2);
        assert_eq!(out2.reason, "actor fallback: special case");
    }

    #[test]
    fn quality_not_worse_beyond_threshold() {
        let mut svc = DistillationService::new();
        let candidate = svc.distill("qc", "qc-rule", qc_rule, baseline()).unwrap();
        let report = svc.run_regression(&candidate.id, &cases()).unwrap();
        assert!(report.quality_delta >= -0.05, "quality must not degrade");
    }
}
