//! Evolution Flywheel v0.2: trace ingestion, pattern detection, candidate
//! generation with evidence windows. Candidates are data-only: they hold no
//! production write authority.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    FlywheelCandidateId, FlywheelPatternId, HumanCorrectionId, TraceRecordId, WorkspaceId,
};
use morn_kernel::time::Timestamp;

/// A single execution trace record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceRecord {
    pub id: TraceRecordId,
    pub workspace_id: WorkspaceId,
    pub run_ref: String,
    pub step: String,
    pub event_type: String, // executed | succeeded | failed | approval | human_correction
    pub outcome: String,
    pub latency_ms: u64,
    pub cost: f64,
    pub principal: String,
    pub created_at: Timestamp,
}

impl TraceRecord {
    pub fn new(
        workspace_id: WorkspaceId,
        run_ref: impl Into<String>,
        step: impl Into<String>,
        event_type: impl Into<String>,
        outcome: impl Into<String>,
    ) -> Self {
        Self {
            id: TraceRecordId::generate_with("trace"),
            workspace_id,
            run_ref: run_ref.into(),
            step: step.into(),
            event_type: event_type.into(),
            outcome: outcome.into(),
            latency_ms: 0,
            cost: 0.0,
            principal: "system".to_string(),
            created_at: Timestamp::now(),
        }
    }
}

/// A human correction event (feeds candidate evidence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanCorrection {
    pub id: HumanCorrectionId,
    pub workspace_id: WorkspaceId,
    pub run_ref: String,
    pub step: String,
    pub correction: String,
    pub created_at: Timestamp,
}

impl HumanCorrection {
    pub fn new(
        workspace_id: WorkspaceId,
        run_ref: impl Into<String>,
        step: impl Into<String>,
        correction: impl Into<String>,
    ) -> Self {
        Self {
            id: HumanCorrectionId::generate_with("hcorr"),
            workspace_id,
            run_ref: run_ref.into(),
            step: step.into(),
            correction: correction.into(),
            created_at: Timestamp::now(),
        }
    }
}

/// Pattern kinds the flywheel can detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PatternKind {
    RepeatedSuccess,
    RepeatedFailure,
    RepeatedHumanCorrection,
    RepeatedApproval,
    HighLatencyStep,
    HighCostStep,
    DeterministicCandidate,
    RedundantHandoff,
    CapabilityGap,
    LowValueSoftwareDependency,
}

/// A detected pattern with its evidence window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub id: FlywheelPatternId,
    pub kind: PatternKind,
    pub affected_work: String,
    pub evidence_window_start: Option<String>,
    pub evidence_window_end: Option<String>,
    pub count: u32,
    pub baseline_quality: f64,
    pub baseline_latency_ms: u64,
    pub baseline_cost: f64,
    pub baseline_human_interventions: u32,
    pub created_at: Timestamp,
}

/// A flywheel candidate (data-only; no production mutation methods).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlywheelCandidate {
    pub id: FlywheelCandidateId,
    pub workspace_id: WorkspaceId,
    pub candidate_type: String, // skill | workflow | harness_patch | deterministic_distillation | workcell | role_suggestion | software_replacement
    pub affected_work: String,
    pub source_evidence_window: Vec<String>,
    pub baseline_metrics: String,
    pub proposed_change: String,
    pub risk: String,
    pub required_evaluations: Vec<String>,
    pub expected_benefit: String,
    pub rollback_plan: String,
    pub status: String,
    pub created_at: Timestamp,
}

/// The evolution flywheel service.
#[derive(Debug, Default)]
pub struct EvolutionFlywheel {
    pub traces: Vec<TraceRecord>,
    pub corrections: Vec<HumanCorrection>,
    pub patterns: Vec<Pattern>,
    pub candidates: Vec<FlywheelCandidate>,
}

impl EvolutionFlywheel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest_trace(&mut self, record: TraceRecord) {
        self.traces.push(record);
    }

    pub fn ingest_human_correction(&mut self, correction: HumanCorrection) {
        self.corrections.push(correction);
    }

    /// Detect patterns from ingested traces. Fixture runs in.
    pub fn detect_patterns(
        &mut self,
        success_threshold: u32,
        latency_threshold_ms: u64,
    ) -> Vec<Pattern> {
        let mut by_step: HashMap<String, Vec<&TraceRecord>> = HashMap::new();
        for t in &self.traces {
            by_step.entry(t.step.clone()).or_default().push(t);
        }
        let mut patterns = Vec::new();
        for (step, records) in by_step {
            let successes = records
                .iter()
                .filter(|r| r.event_type == "succeeded")
                .count() as u32;
            let failures = records.iter().filter(|r| r.event_type == "failed").count() as u32;
            let approvals = records
                .iter()
                .filter(|r| r.event_type == "approval")
                .count() as u32;
            let avg_latency =
                records.iter().map(|r| r.latency_ms).sum::<u64>() / records.len().max(1) as u64;
            let total_cost: f64 = records.iter().map(|r| r.cost).sum();
            let corrections = self.corrections.iter().filter(|c| c.step == step).count() as u32;

            let start = records.iter().map(|r| r.id.to_string()).min();
            let end = records.iter().map(|r| r.id.to_string()).max();
            let evidence = |kind: PatternKind, count: u32| Pattern {
                id: FlywheelPatternId::generate_with("pat"),
                kind,
                affected_work: step.clone(),
                evidence_window_start: start.clone(),
                evidence_window_end: end.clone(),
                count,
                baseline_quality: successes as f64 / records.len().max(1) as f64,
                baseline_latency_ms: avg_latency,
                baseline_cost: total_cost,
                baseline_human_interventions: corrections + approvals,
                created_at: Timestamp::now(),
            };

            if successes >= success_threshold {
                patterns.push(evidence(PatternKind::RepeatedSuccess, successes));
            }
            if failures >= 1 {
                patterns.push(evidence(PatternKind::RepeatedFailure, failures));
            }
            if corrections >= 1 {
                patterns.push(evidence(PatternKind::RepeatedHumanCorrection, corrections));
            }
            if approvals >= success_threshold {
                patterns.push(evidence(PatternKind::RepeatedApproval, approvals));
            }
            if avg_latency >= latency_threshold_ms {
                patterns.push(evidence(PatternKind::HighLatencyStep, records.len() as u32));
            }
            if total_cost >= 100.0 {
                patterns.push(evidence(PatternKind::HighCostStep, records.len() as u32));
            }
        }
        self.patterns = patterns.clone();
        patterns
    }

    /// Generate candidates from detected patterns. Candidates are data-only.
    pub fn generate_candidates(&mut self, workspace_id: &WorkspaceId) -> Vec<FlywheelCandidate> {
        let mut candidates = Vec::new();
        for p in &self.patterns {
            let (candidate_type, change, risk, benefit) = match p.kind {
                PatternKind::RepeatedSuccess => (
                    "deterministic_distillation".to_string(),
                    format!(
                        "distill repeated stable step '{}' into a deterministic program with actor fallback",
                        p.affected_work
                    ),
                    "low".to_string(),
                    "lower cost/latency, stable quality, reduced human load".to_string(),
                ),
                PatternKind::RepeatedFailure => (
                    "harness_patch".to_string(),
                    format!("patch harness/tooling for failing step '{}'", p.affected_work),
                    "medium".to_string(),
                    "raise reliability".to_string(),
                ),
                PatternKind::RepeatedHumanCorrection => (
                    "skill".to_string(),
                    format!(
                        "encode repeated human corrections on '{}' into skill guidance",
                        p.affected_work
                    ),
                    "medium".to_string(),
                    "fewer human interventions".to_string(),
                ),
                PatternKind::RepeatedApproval => (
                    "workflow".to_string(),
                    format!("reduce redundant approvals on '{}' via policy", p.affected_work),
                    "medium".to_string(),
                    "shorter cycle time".to_string(),
                ),
                PatternKind::HighLatencyStep => (
                    "workflow".to_string(),
                    format!("optimize high-latency step '{}'", p.affected_work),
                    "low".to_string(),
                    "lower cycle time".to_string(),
                ),
                PatternKind::HighCostStep => (
                    "workflow".to_string(),
                    format!("optimize high-cost step '{}'", p.affected_work),
                    "low".to_string(),
                    "lower cost".to_string(),
                ),
                _ => continue,
            };
            let evidence_window = vec![
                p.evidence_window_start.clone().unwrap_or_default(),
                p.evidence_window_end.clone().unwrap_or_default(),
            ];
            let candidate = FlywheelCandidate {
                id: FlywheelCandidateId::generate_with("fwc"),
                workspace_id: workspace_id.clone(),
                candidate_type,
                affected_work: p.affected_work.clone(),
                source_evidence_window: evidence_window,
                baseline_metrics: format!(
                    "quality={:.2} latency_ms={} cost={:.2} human={}",
                    p.baseline_quality,
                    p.baseline_latency_ms,
                    p.baseline_cost,
                    p.baseline_human_interventions
                ),
                proposed_change: change,
                risk,
                required_evaluations: vec![
                    "regression".to_string(),
                    "acceptance".to_string(),
                    "policy".to_string(),
                ],
                expected_benefit: benefit,
                rollback_plan: format!("rollback {} to previous workflow version", p.affected_work),
                status: "proposed".to_string(),
                created_at: Timestamp::now(),
            };
            candidates.push(candidate.clone());
            self.candidates.push(candidate);
        }
        candidates
    }

    /// A candidate may not mutate production: this method only returns the
    /// candidate's plan as evidence for a human/evolution review.
    pub fn candidate_plan(&self, id: &FlywheelCandidateId) -> Result<&FlywheelCandidate> {
        self.candidates
            .iter()
            .find(|c| c.id == *id)
            .ok_or_else(|| Error::not_found(format!("candidate {id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(
        ws: &WorkspaceId,
        step: &str,
        successes: u32,
        failures: u32,
        corrections: u32,
    ) -> EvolutionFlywheel {
        let mut fw = EvolutionFlywheel::new();
        for i in 0..successes {
            fw.ingest_trace(TraceRecord::new(
                ws.clone(),
                format!("run-{i}"),
                step,
                "succeeded",
                "ok",
            ));
        }
        for i in 0..failures {
            fw.ingest_trace(TraceRecord::new(
                ws.clone(),
                format!("run-f{i}"),
                step,
                "failed",
                "error",
            ));
        }
        for i in 0..corrections {
            fw.ingest_human_correction(HumanCorrection::new(
                ws.clone(),
                format!("run-c{i}"),
                step,
                "fix formatting",
            ));
        }
        fw
    }

    #[test]
    fn pattern_detection_from_fixture_runs() {
        let ws = WorkspaceId::generate();
        let mut fw = seed(&ws, "qc", 5, 0, 0);
        let patterns = fw.detect_patterns(3, 10_000);
        assert!(patterns
            .iter()
            .any(|p| p.kind == PatternKind::RepeatedSuccess));
        assert!(patterns.iter().any(|p| p.affected_work == "qc"));
        assert_eq!(
            patterns
                .iter()
                .find(|p| p.kind == PatternKind::RepeatedSuccess)
                .unwrap()
                .count,
            5
        );
    }

    #[test]
    fn candidate_links_source_evidence() {
        let ws = WorkspaceId::generate();
        let mut fw = seed(&ws, "qc", 5, 0, 0);
        fw.detect_patterns(3, 10_000);
        let candidates = fw.generate_candidates(&ws);
        let det = candidates
            .iter()
            .find(|c| c.candidate_type == "deterministic_distillation")
            .expect("distillation candidate");
        assert!(!det.source_evidence_window.is_empty());
        assert!(det.proposed_change.contains("qc"));
        assert!(det.rollback_plan.contains("rollback"));
    }

    #[test]
    fn human_correction_contributes_evidence() {
        let ws = WorkspaceId::generate();
        let mut fw = seed(&ws, "review", 3, 0, 4);
        fw.detect_patterns(3, 10_000);
        let candidates = fw.generate_candidates(&ws);
        assert!(candidates.iter().any(|c| c.candidate_type == "skill"));
    }

    #[test]
    fn candidate_cannot_mutate_production() {
        // FlywheelCandidate has no mutation methods and the flywheel exposes no
        // production write path: it only records plans for human/evolution review.
        let ws = WorkspaceId::generate();
        let mut fw = seed(&ws, "qc", 5, 0, 0);
        fw.detect_patterns(3, 10_000);
        fw.generate_candidates(&ws);
        let cid = fw.candidates[0].id.clone();
        let plan = fw.candidate_plan(&cid).unwrap();
        assert!(!plan.proposed_change.is_empty());
        assert_eq!(
            fw.candidates.len(),
            fw.candidates
                .iter()
                .filter(|c| c.status == "proposed")
                .count()
        );
    }
}
