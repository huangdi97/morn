//! outcome — ObservedOutcome / AcceptanceDecision / ValueAssessment.
//!
//! INVARIANT: an ObservedOutcome comes from an identifiable source (external
//! system, sensor, human observation, validated computation). An Agent
//! self-report of success is never an outcome. ExecutionFinished != Delivered
//! != OutcomeObserved != Accepted.

use serde::{Deserialize, Serialize};

/// One measured value with an explicit unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeMetric {
    pub key: String,
    pub value: f64,
    pub unit: String,
}

impl OutcomeMetric {
    pub fn new(key: &str, value: f64, unit: &str) -> Self {
        Self {
            key: key.to_string(),
            value,
            unit: unit.to_string(),
        }
    }
}

/// Where an outcome came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutcomeSource {
    ExternalSystem,
    Sensor,
    HumanObservation,
    ValidatedComputation,
}

/// ObservedOutcome — the observed reality, from an identifiable source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedOutcome {
    pub id: String,
    pub work_ref: String,
    pub source: OutcomeSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_time: Option<String>,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<OutcomeMetric>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
}

impl ObservedOutcome {
    pub fn from_source(id: &str, work_ref: &str, source: OutcomeSource) -> Self {
        Self {
            id: id.to_string(),
            work_ref: work_ref.to_string(),
            source,
            source_ref: None,
            valid_time: None,
            recorded_at: crate::work::now_iso(),
            metrics: Vec::new(),
            notes: None,
            supersedes: None,
        }
    }

    pub fn with_metric(mut self, metric: OutcomeMetric) -> Self {
        self.metrics.push(metric);
        self
    }
}

/// AcceptanceDecision — the business judgment over an outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcceptanceType {
    Accept,
    Reject,
    Conditional,
}

/// Acceptance is a separate decision from execution finishing; multiple roles
/// may be required to accept (multi-role acceptance requirements).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceDecision {
    pub id: String,
    pub work_ref: String,
    pub decision: AcceptanceType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome_refs: Vec<String>,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub decided_at: String,
}

impl AcceptanceDecision {
    pub fn new(id: &str, work_ref: &str, decision: AcceptanceType, actor: &str) -> Self {
        Self {
            id: id.to_string(),
            work_ref: work_ref.to_string(),
            decision,
            criteria_refs: Vec::new(),
            outcome_refs: Vec::new(),
            actor: actor.to_string(),
            principal: None,
            reason: None,
            decided_at: crate::work::now_iso(),
        }
    }
}

/// ValueAssessment — value is measured in accepted outcomes, not agent runs or
/// token counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueAssessment {
    pub id: String,
    pub work_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_duration_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_runtime_cost: Option<crate::work::CostAmount>,
    #[serde(default)]
    pub retries: i64,
    #[serde(default)]
    pub unknown_outcomes: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_comparison_ref: Option<String>,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub reopened: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kpis: Vec<OutcomeMetric>,
    pub recorded_at: String,
}

impl ValueAssessment {
    pub fn new(id: &str, work_ref: &str) -> Self {
        Self {
            id: id.to_string(),
            work_ref: work_ref.to_string(),
            total_duration_minutes: None,
            human_minutes: None,
            model_runtime_cost: None,
            retries: 0,
            unknown_outcomes: 0,
            baseline_comparison_ref: None,
            accepted: false,
            reopened: false,
            kpis: Vec::new(),
            recorded_at: crate::work::now_iso(),
        }
    }
}
