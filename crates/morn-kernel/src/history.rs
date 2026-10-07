//! Bitemporal, non-destructive historical facts.
//!
//! Current projections are mutable, but corrections to facts that influenced
//! work are explicit. `valid_time` says when the fact applied in the outside
//! world; `recorded_at` says when Morn learned/recorded it. Corrections append
//! a new fact and relation rather than silently rewriting the old payload.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::ids::Id;
use crate::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HistoricalFactTag;
pub type HistoricalFactId = Id<HistoricalFactTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HistoricalFactStatus {
    Active,
    Superseded,
    Retracted,
    Redacted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalFact {
    pub id: HistoricalFactId,
    pub subject_ref: String,
    pub fact_type: String,
    pub payload: Value,
    pub source_ref: String,
    pub valid_time: Timestamp,
    pub recorded_at: Timestamp,
    pub status: HistoricalFactStatus,
    pub supersedes: Option<HistoricalFactId>,
    pub reason: Option<String>,
}

impl HistoricalFact {
    pub fn observed(
        subject_ref: impl Into<String>,
        fact_type: impl Into<String>,
        payload: Value,
        source_ref: impl Into<String>,
        valid_time: Timestamp,
    ) -> Self {
        Self {
            id: HistoricalFactId::generate_with("fact"),
            subject_ref: subject_ref.into(),
            fact_type: fact_type.into(),
            payload,
            source_ref: source_ref.into(),
            valid_time,
            recorded_at: Timestamp::now(),
            status: HistoricalFactStatus::Active,
            supersedes: None,
            reason: None,
        }
    }

    pub fn superseding(
        prior: &HistoricalFact,
        payload: Value,
        source_ref: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: HistoricalFactId::generate_with("fact"),
            subject_ref: prior.subject_ref.clone(),
            fact_type: prior.fact_type.clone(),
            payload,
            source_ref: source_ref.into(),
            valid_time: prior.valid_time,
            recorded_at: Timestamp::now(),
            status: HistoricalFactStatus::Active,
            supersedes: Some(prior.id.clone()),
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HistoricalFactLog {
    pub facts: Vec<HistoricalFact>,
}

impl HistoricalFactLog {
    pub fn append(&mut self, fact: HistoricalFact) -> Result<()> {
        if self.facts.iter().any(|existing| existing.id == fact.id) {
            return Err(Error::conflict(format!(
                "historical fact {} already exists",
                fact.id
            )));
        }
        if let Some(prior_id) = &fact.supersedes {
            let prior = self
                .facts
                .iter_mut()
                .find(|existing| existing.id == *prior_id)
                .ok_or_else(|| Error::not_found(format!("historical fact {prior_id}")))?;
            if prior.subject_ref != fact.subject_ref || prior.fact_type != fact.fact_type {
                return Err(Error::validation(
                    "superseding fact must keep subject and fact type",
                ));
            }
            prior.status = HistoricalFactStatus::Superseded;
        }
        self.facts.push(fact);
        Ok(())
    }

    pub fn retract(&mut self, id: &HistoricalFactId, reason: impl Into<String>) -> Result<()> {
        let fact = self
            .facts
            .iter_mut()
            .find(|existing| existing.id == *id)
            .ok_or_else(|| Error::not_found(format!("historical fact {id}")))?;
        fact.status = HistoricalFactStatus::Retracted;
        fact.reason = Some(reason.into());
        Ok(())
    }

    pub fn redact(&mut self, id: &HistoricalFactId, reason: impl Into<String>) -> Result<()> {
        let fact = self
            .facts
            .iter_mut()
            .find(|existing| existing.id == *id)
            .ok_or_else(|| Error::not_found(format!("historical fact {id}")))?;
        fact.status = HistoricalFactStatus::Redacted;
        fact.payload = Value::Null;
        fact.reason = Some(reason.into());
        Ok(())
    }

    pub fn current_for(&self, subject_ref: &str, fact_type: &str) -> Option<&HistoricalFact> {
        self.facts.iter().rev().find(|fact| {
            fact.subject_ref == subject_ref
                && fact.fact_type == fact_type
                && fact.status == HistoricalFactStatus::Active
        })
    }

    pub fn as_recorded_at(
        &self,
        subject_ref: &str,
        fact_type: &str,
        cutoff: Timestamp,
    ) -> Option<&HistoricalFact> {
        self.facts.iter().rev().find(|fact| {
            fact.subject_ref == subject_ref
                && fact.fact_type == fact_type
                && fact.recorded_at <= cutoff
                && matches!(
                    fact.status,
                    HistoricalFactStatus::Active | HistoricalFactStatus::Superseded
                )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn correction_preserves_what_was_known_when() {
        let valid_time = Timestamp::from_millis(1_000);
        let mut log = HistoricalFactLog::default();
        let original = HistoricalFact::observed(
            "machine:CNC-17",
            "temperature",
            json!({"celsius":82}),
            "sensor:A",
            valid_time,
        );
        let original_id = original.id.clone();
        let before_correction = original.recorded_at;
        log.append(original).unwrap();

        let replacement = HistoricalFact::superseding(
            log.current_for("machine:CNC-17", "temperature").unwrap(),
            json!({"celsius":63}),
            "sensor:B",
            "sensor A calibration failure",
        );
        let replacement_id = replacement.id.clone();
        log.append(replacement).unwrap();

        let old = log
            .facts
            .iter()
            .find(|fact| fact.id == original_id)
            .unwrap();
        assert_eq!(old.status, HistoricalFactStatus::Superseded);
        assert_eq!(old.payload["celsius"], 82);

        let current = log.current_for("machine:CNC-17", "temperature").unwrap();
        assert_eq!(current.id, replacement_id);
        assert_eq!(current.payload["celsius"], 63);

        let past = log
            .as_recorded_at("machine:CNC-17", "temperature", before_correction)
            .unwrap();
        assert_eq!(past.payload["celsius"], 82);
    }

    #[test]
    fn redaction_removes_payload_but_keeps_auditable_tombstone() {
        let mut log = HistoricalFactLog::default();
        let fact = HistoricalFact::observed(
            "person:1",
            "private-note",
            json!({"secret":"remove-me"}),
            "user-input",
            Timestamp::now(),
        );
        let id = fact.id.clone();
        log.append(fact).unwrap();
        log.redact(&id, "privacy request").unwrap();

        let redacted = log.facts.iter().find(|fact| fact.id == id).unwrap();
        assert_eq!(redacted.status, HistoricalFactStatus::Redacted);
        assert_eq!(redacted.payload, Value::Null);
        assert_eq!(redacted.reason.as_deref(), Some("privacy request"));
    }
}
