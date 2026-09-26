//! common — ObjectRef / SourceRef / Provenance / HistoricalRecord.
//!
//! INVARIANT: historical facts are non-destructive. A record may supersede,
//! retract or redact another, but a committed fact is never silently
//! rewritten. Dual-time semantics: `valid_time` is when the fact held in
//! reality; `recorded_at` is when the system learned it.

use serde::{Deserialize, Serialize};

/// Reference to any object (internal or external).
///
/// `kind` is open-world by contract: any external system may be referenced
/// (asset, work, work_order, capability, site, ...). A closed enum would break
/// the open-world reference model, so the kind stays a documented string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRef {
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl ObjectRef {
    pub fn new(kind: &str, id: &str) -> Self {
        Self {
            kind: kind.to_string(),
            id: id.to_string(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: SourceRef) -> Self {
        self.source = Some(source);
        self
    }
}

/// Binding to the external system that owns a fact.
///
/// INVARIANT: the external system remains the source of truth; Morn records
/// observations of it, it does not own world facts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRef {
    pub system: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl SourceRef {
    pub fn new(system: &str) -> Self {
        Self {
            system: system.to_string(),
            external_id: None,
            uri: None,
        }
    }

    pub fn with_external_id(mut self, external_id: &str) -> Self {
        self.external_id = Some(external_id.to_string());
        self
    }
}

/// Who recorded a fact, when, and why.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub actor: String,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Provenance {
    pub fn now(actor: &str) -> Self {
        Self {
            actor: actor.to_string(),
            recorded_at: crate::work::now_iso(),
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

/// Non-destructive historical envelope around any payload.
///
/// COMPATIBILITY: payloads are stored as `serde_json::Value` so any semantic
/// record can be wrapped without duplicating its type as a second ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalRecord {
    pub id: String,
    pub record_type: String,
    pub payload: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retracts: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_time: Option<String>,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<String>,
}

impl HistoricalRecord {
    /// Records a new fact.
    pub fn record(
        id: &str,
        record_type: &str,
        payload: serde_json::Value,
        valid_time: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            record_type: record_type.to_string(),
            payload,
            supersedes: None,
            retracts: None,
            valid_time: Some(valid_time.to_string()),
            recorded_at: crate::work::now_iso(),
            provenance: None,
            contract_version: Some(crate::work::SEMANTIC_CONTRACT_VERSION.to_string()),
        }
    }

    /// Supersedes an earlier record. The earlier record stays queryable; the
    /// current projection uses the new record, but history explains why the
    /// old decision was made at the time.
    pub fn supersede(
        id: &str,
        record_type: &str,
        supersedes: &str,
        payload: serde_json::Value,
        valid_time: &str,
    ) -> Self {
        let mut record = Self::record(id, record_type, payload, valid_time);
        record.supersedes = Some(supersedes.to_string());
        record
    }

    /// Retracts an earlier record without replacing it.
    pub fn retract(id: &str, record_type: &str, retracts: &str, reason: &str) -> Self {
        let recorded = crate::work::now_iso();
        let mut record = Self::record(id, record_type, serde_json::json!({}), &recorded);
        record.retracts = Some(retracts.to_string());
        record.provenance = Some(Provenance::now("morn").with_reason(reason));
        record
    }
}
