//! world — Observation / Situation.
//!
//! INVARIANT: external systems remain the source of truth; Morn observes the
//! world, it does not own world facts. Dual-time semantics apply to
//! observations: a corrected reading supersedes the old fact instead of
//! overwriting it, so history can still explain why an alarm fired.

use crate::work::ObjectRef;
use serde::{Deserialize, Serialize};

/// One observed fact from an identifiable source, with explicit unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub source: crate::work::SourceRef,
    pub kind: String,
    pub value: f64,
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_ref: Option<ObjectRef>,
    pub valid_time: String,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub invalidated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Observation {
    /// Records a fresh observation.
    pub fn record(
        id: &str,
        source: crate::work::SourceRef,
        kind: &str,
        value: f64,
        unit: &str,
        valid_time: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            source,
            kind: kind.to_string(),
            value,
            unit: unit.to_string(),
            asset_ref: None,
            valid_time: valid_time.to_string(),
            recorded_at: crate::work::now_iso(),
            supersedes: None,
            invalidated: false,
            notes: None,
        }
    }

    /// Produces a corrected observation that supersedes this one. This record
    /// stays queryable; the projection uses the new fact.
    pub fn supersede_with(&self, new_id: &str, new_value: f64, reason: &str) -> Self {
        let mut corrected = Self::record(
            new_id,
            self.source.clone(),
            &self.kind,
            new_value,
            &self.unit,
            &self.valid_time,
        );
        corrected.supersedes = Some(self.id.clone());
        corrected.notes = Some(reason.to_string());
        corrected
    }
}

/// Situation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SituationStatus {
    Open,
    Closed,
    Superseded,
}

/// A Situation groups observations into a coherent context (e.g. a possible
/// equipment degradation). It does not start an Agent; work creation decides
/// that.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Situation {
    pub id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_refs: Vec<ObjectRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_order: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observation_ids: Vec<String>,
    pub status: SituationStatus,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Situation {
    pub fn open(id: &str, kind: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: kind.to_string(),
            work_ref: None,
            asset_refs: Vec::new(),
            current_order: None,
            batch: None,
            observation_ids: Vec::new(),
            status: SituationStatus::Open,
            created_at: crate::work::now_iso(),
            notes: None,
        }
    }

    pub fn with_asset(mut self, asset: ObjectRef) -> Self {
        self.asset_refs.push(asset);
        self
    }

    pub fn with_observation(mut self, observation_id: &str) -> Self {
        self.observation_ids.push(observation_id.to_string());
        self
    }
}
