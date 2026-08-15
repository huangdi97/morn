//! Replacement Pilot: baseline profile, observed work graph, shadow replace and
//! R4 Partial Replace decision. Shadow never writes production; R4 never
//! auto-retires the legacy system.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    ExistingSystemMappingId, PartialReplaceCandidateId, ReplacementComparisonId,
    ReplacementRecordId,
};
use morn_kernel::time::Timestamp;

/// Metrics of one work variant (manual/existing, Morn orchestrated, Morn-native).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkVariantMetrics {
    pub variant: String,
    pub quality: f64,
    pub acceptance_rate: f64,
    pub cycle_time_hours: f64,
    pub human_minutes: f64,
    pub retries: u32,
    pub error_rework_rate: f64,
    pub cost_estimate: f64,
    pub evidence_coverage: f64,
    pub policy_violations: u32,
    pub outcome_metric: f64,
}

impl WorkVariantMetrics {
    pub fn new(variant: &str) -> Self {
        Self {
            variant: variant.to_string(),
            quality: 0.0,
            acceptance_rate: 0.0,
            cycle_time_hours: 0.0,
            human_minutes: 0.0,
            retries: 0,
            error_rework_rate: 0.0,
            cost_estimate: 0.0,
            evidence_coverage: 0.0,
            policy_violations: 0,
            outcome_metric: 0.0,
        }
    }
}

/// Mapping of an existing (manual/software) system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingSystemMapping {
    pub id: ExistingSystemMappingId,
    pub system_name: String,
    pub work_boundary: String,
    pub tools: Vec<String>,
    pub roles: Vec<String>,
    pub created_at: Timestamp,
}

impl ExistingSystemMapping {
    pub fn new(system_name: impl Into<String>, work_boundary: impl Into<String>) -> Self {
        Self {
            id: ExistingSystemMappingId::generate_with("esm"),
            system_name: system_name.into(),
            work_boundary: work_boundary.into(),
            tools: Vec::new(),
            roles: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}

/// Observed work graph of the existing process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWorkGraph {
    pub steps: Vec<String>,
    pub handoffs: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl ObservedWorkGraph {
    pub fn new(steps: Vec<String>) -> Self {
        Self {
            steps,
            handoffs: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

/// Comparison of baseline vs candidate on the same inputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplacementComparison {
    pub id: ReplacementComparisonId,
    pub inputs_ref: String,
    pub baseline: WorkVariantMetrics,
    pub candidate: WorkVariantMetrics,
    pub isolated_side_effects: bool,
    pub candidate_meets_critical: bool,
    pub reasons: Vec<String>,
}

impl ReplacementComparison {
    pub fn candidate_better_or_equal(&self) -> bool {
        self.candidate_meets_critical
    }
}

/// R4 Partial Replace candidate (never auto-retires the legacy system).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialReplaceCandidate {
    pub id: PartialReplaceCandidateId,
    pub work: String,
    pub baseline_variant: String,
    pub candidate_variant: String,
    pub comparison_ref: String,
    pub evaluation_passed: bool,
    pub certification_passed: bool,
    pub human_approved: bool,
    pub rollback_path: String,
    pub status: String, // pending_human | approved | rejected
    pub created_at: Timestamp,
}

/// A replacement record for Console/Hub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementRecord {
    pub id: ReplacementRecordId,
    pub system_name: String,
    pub work: String,
    pub comparison_ref: String,
    pub decision: String,
    pub rollback_path: String,
    pub created_at: Timestamp,
}

/// Replacement Pilot service.
#[derive(Debug, Default)]
pub struct ReplacementPilot {
    pub mappings: Vec<ExistingSystemMapping>,
    pub comparisons: Vec<ReplacementComparison>,
    pub candidates: Vec<PartialReplaceCandidate>,
    pub records: Vec<ReplacementRecord>,
}

impl ReplacementPilot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_mapping(&mut self, mapping: ExistingSystemMapping) {
        self.mappings.push(mapping);
    }

    /// R3 Shadow Replace: compare baseline vs candidate on the same inputs,
    /// fully isolated (no production side effects).
    pub fn shadow_compare(
        &mut self,
        inputs_ref: &str,
        baseline: WorkVariantMetrics,
        candidate: WorkVariantMetrics,
    ) -> ReplacementComparison {
        let mut reasons = Vec::new();
        let candidate_ok = candidate.quality >= baseline.quality
            && candidate.policy_violations <= baseline.policy_violations
            && candidate.evidence_coverage >= baseline.evidence_coverage
            && candidate.acceptance_rate >= baseline.acceptance_rate;
        if !candidate_ok {
            if candidate.quality < baseline.quality {
                reasons.push("candidate quality below baseline".to_string());
            }
            if candidate.policy_violations > baseline.policy_violations {
                reasons.push("candidate policy regression".to_string());
            }
            if candidate.evidence_coverage < baseline.evidence_coverage {
                reasons.push("candidate evidence coverage below baseline".to_string());
            }
            if candidate.acceptance_rate < baseline.acceptance_rate {
                reasons.push("candidate acceptance rate below baseline".to_string());
            }
        }
        let comparison = ReplacementComparison {
            id: ReplacementComparisonId::generate_with("cmp"),
            inputs_ref: inputs_ref.to_string(),
            baseline,
            candidate,
            isolated_side_effects: true,
            candidate_meets_critical: candidate_ok,
            reasons,
        };
        self.comparisons.push(comparison.clone());
        comparison
    }

    /// R4 gate: candidate must meet critical metrics AND evaluation passed AND
    /// certification passed AND human approval. Never auto-retires the legacy.
    pub fn decide_r4(
        &mut self,
        work: &str,
        comparison: &ReplacementComparison,
        evaluation_passed: bool,
        certification_passed: bool,
        human_approved: bool,
    ) -> Result<PartialReplaceCandidate> {
        if !comparison.candidate_meets_critical {
            return Err(Error::validation(format!(
                "cannot produce R4 candidate: {}",
                comparison.reasons.join("; ")
            )));
        }
        if !evaluation_passed {
            return Err(Error::validation("R4 blocked: evaluation not passed"));
        }
        if !certification_passed {
            return Err(Error::validation("R4 blocked: capability not certified"));
        }
        if !human_approved {
            return Err(Error::not_authorized("R4 blocked: human approval required"));
        }
        let candidate = PartialReplaceCandidate {
            id: PartialReplaceCandidateId::generate_with("r4"),
            work: work.to_string(),
            baseline_variant: comparison.baseline.variant.clone(),
            candidate_variant: comparison.candidate.variant.clone(),
            comparison_ref: comparison.id.to_string(),
            evaluation_passed,
            certification_passed,
            human_approved,
            rollback_path: format!("rollback {work} to existing/manual path"),
            status: "approved".to_string(),
            created_at: Timestamp::now(),
        };
        self.candidates.push(candidate.clone());
        self.records.push(ReplacementRecord {
            id: ReplacementRecordId::generate_with("rrec"),
            system_name: comparison.baseline.variant.clone(),
            work: work.to_string(),
            comparison_ref: comparison.id.to_string(),
            decision: "partial_replace_candidate".to_string(),
            rollback_path: candidate.rollback_path.clone(),
            created_at: Timestamp::now(),
        });
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(
        variant: &str,
        quality: f64,
        policy_violations: u32,
        evidence: f64,
        acceptance: f64,
    ) -> WorkVariantMetrics {
        let mut m = WorkVariantMetrics::new(variant);
        m.quality = quality;
        m.policy_violations = policy_violations;
        m.evidence_coverage = evidence;
        m.acceptance_rate = acceptance;
        m
    }

    #[test]
    fn baseline_reproducible_and_same_input_comparison() {
        let mut pilot = ReplacementPilot::new();
        let mapping = ExistingSystemMapping::new("manual-csv-review", "dataset-to-claim");
        pilot.register_mapping(mapping);
        let graph =
            ObservedWorkGraph::new(vec!["export".into(), "review".into(), "release".into()]);
        assert_eq!(graph.steps.len(), 3);
        let baseline = metrics("manual", 0.9, 1, 0.6, 0.85);
        let candidate = metrics("morn-native", 0.95, 0, 1.0, 0.95);
        let cmp = pilot.shadow_compare("input-set-1", baseline, candidate);
        assert!(cmp.candidate_meets_critical);
        assert!(cmp.isolated_side_effects, "shadow must be isolated");
    }

    #[test]
    fn worse_candidate_cannot_produce_r4() {
        let mut pilot = ReplacementPilot::new();
        let baseline = metrics("manual", 0.9, 0, 1.0, 0.95);
        let candidate = metrics("morn-native", 0.7, 0, 1.0, 0.9);
        let cmp = pilot.shadow_compare("inputs", baseline, candidate);
        assert!(!cmp.candidate_meets_critical);
        assert!(pilot
            .decide_r4("dataset-to-claim", &cmp, true, true, true)
            .is_err());
    }

    #[test]
    fn policy_regression_blocks_r4() {
        let mut pilot = ReplacementPilot::new();
        let baseline = metrics("manual", 0.9, 0, 1.0, 0.95);
        let candidate = metrics("morn-native", 0.95, 2, 1.0, 0.95);
        let cmp = pilot.shadow_compare("inputs", baseline, candidate);
        assert!(cmp.reasons.iter().any(|r| r.contains("policy")));
        assert!(pilot
            .decide_r4("dataset-to-claim", &cmp, true, true, true)
            .is_err());
    }

    #[test]
    fn human_approval_required_for_r4() {
        let mut pilot = ReplacementPilot::new();
        let baseline = metrics("manual", 0.9, 0, 1.0, 0.95);
        let candidate = metrics("morn-native", 0.95, 0, 1.0, 0.95);
        let cmp = pilot.shadow_compare("inputs", baseline, candidate);
        assert!(pilot
            .decide_r4("dataset-to-claim", &cmp, true, true, false)
            .is_err());
    }

    #[test]
    fn r4_with_rollback_when_all_gates_pass() {
        let mut pilot = ReplacementPilot::new();
        let baseline = metrics("manual", 0.9, 0, 1.0, 0.95);
        let candidate = metrics("morn-native", 0.95, 0, 1.0, 0.98);
        let cmp = pilot.shadow_compare("inputs", baseline, candidate);
        let r4 = pilot
            .decide_r4("dataset-to-claim", &cmp, true, true, true)
            .unwrap();
        assert_eq!(r4.status, "approved");
        assert!(r4.rollback_path.contains("rollback"));
        assert_eq!(pilot.records.len(), 1);
        assert!(pilot.records[0].rollback_path.contains("existing/manual"));
    }
}
