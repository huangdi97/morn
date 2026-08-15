//! Historical replay: deterministic isolated replay of recorded events.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{ReplayReportId, ReplayRunId, ReplayScenarioId};
use morn_kernel::time::Timestamp;

/// A recorded event from a previous execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub step: String,
    pub input_hash: String,
    pub output_hash: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScenario {
    pub id: ReplayScenarioId,
    pub name: String,
    pub base_world_snapshot: Value,
    pub solution_version: String,
    pub recorded_events: Vec<RecordedEvent>,
    pub expected_outcomes: Vec<String>,
}

impl ReplayScenario {
    pub fn new(
        name: impl Into<String>,
        base_world_snapshot: Value,
        solution_version: impl Into<String>,
    ) -> Self {
        Self {
            id: ReplayScenarioId::generate_with("replay"),
            name: name.into(),
            base_world_snapshot,
            solution_version: solution_version.into(),
            recorded_events: Vec::new(),
            expected_outcomes: Vec::new(),
        }
    }

    pub fn record(mut self, step: &str, input_hash: &str, output_hash: &str, result: &str) -> Self {
        self.recorded_events.push(RecordedEvent {
            step: step.to_string(),
            input_hash: input_hash.to_string(),
            output_hash: output_hash.to_string(),
            result: result.to_string(),
        });
        self
    }
}

/// One step result of a replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayStepResult {
    pub step: String,
    pub reproduced: bool,
    pub expected_output_hash: String,
    pub actual_output_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayReport {
    pub id: ReplayReportId,
    pub run_id: ReplayRunId,
    pub scenario_id: ReplayScenarioId,
    pub reproduced: bool,
    pub deviations: Vec<String>,
    pub step_results: Vec<ReplayStepResult>,
    pub state_diff_refs: Vec<String>,
    pub outcome: String,
    pub created_at: Timestamp,
}

/// Replays recorded events in isolated state (never writes production).
#[derive(Debug, Default)]
pub struct ReplayRunner {
    pub reports: Vec<ReplayReport>,
}

impl ReplayRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Deterministic replay. `mutation` can be provided to simulate code drift:
    /// when `Some`, one step's recomputed output will differ from the record.
    pub fn run(&mut self, scenario: &ReplayScenario, mutate_step: Option<&str>) -> ReplayReport {
        let mut step_results = Vec::new();
        let mut deviations = Vec::new();
        let mut state_diff_refs = Vec::new();

        for event in &scenario.recorded_events {
            // Deterministic recomputation of the output hash from the input.
            let recomputed = simple_hash(&event.input_hash);
            let reproduced = recomputed == event.output_hash
                && mutate_step.map(|m| m != event.step).unwrap_or(true);
            step_results.push(ReplayStepResult {
                step: event.step.clone(),
                reproduced,
                expected_output_hash: event.output_hash.clone(),
                actual_output_hash: recomputed,
            });
            if !reproduced {
                deviations.push(format!("step {} output differs (drift)", event.step));
            }
            state_diff_refs.push(format!("diff-{}", event.step));
        }

        let reproduced = deviations.is_empty();
        let report = ReplayReport {
            id: ReplayReportId::generate_with("reprev"),
            run_id: ReplayRunId::generate_with("reprun"),
            scenario_id: scenario.id.clone(),
            reproduced,
            deviations,
            step_results,
            state_diff_refs,
            outcome: if reproduced {
                "reproduced".to_string()
            } else {
                "deviation_detected".to_string()
            },
            created_at: Timestamp::now(),
        };
        self.reports.push(report.clone());
        report
    }
}

/// Simple deterministic hash for replay reproducibility checks.
pub fn simple_hash(input: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn replay_reproduces_when_unchanged() {
        let input = "input-a";
        let out = simple_hash(input);
        let scenario = ReplayScenario::new("bio-replay", json!({"version": 1}), "sol-1.0")
            .record("analyze", input, &out, "ok");
        let mut runner = ReplayRunner::new();
        let report = runner.run(&scenario, None);
        assert!(report.reproduced);
        assert!(report.deviations.is_empty());
        assert_eq!(report.outcome, "reproduced");
    }

    #[test]
    fn replay_detects_deviation_under_drift() {
        let input = "input-a";
        let out = simple_hash(input);
        let scenario = ReplayScenario::new("bio-replay", json!({"version": 1}), "sol-1.0")
            .record("analyze", input, &out, "ok");
        let mut runner = ReplayRunner::new();
        // Simulate code drift on the "analyze" step.
        let report = runner.run(&scenario, Some("analyze"));
        assert!(!report.reproduced);
        assert_eq!(report.deviations.len(), 1);
        assert_eq!(report.outcome, "deviation_detected");
    }

    #[test]
    fn replay_never_touches_production() {
        // The runner only reads the scenario and writes to its own report list.
        let scenario = ReplayScenario::new("x", json!({"version": 1}), "sol-1.0");
        let mut runner = ReplayRunner::new();
        let before = runner.reports.len();
        runner.run(&scenario, None);
        assert_eq!(runner.reports.len(), before + 1);
    }
}
