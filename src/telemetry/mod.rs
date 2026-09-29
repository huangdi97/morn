//! telemetry — Morn semantic conventions + CloudEvents envelope (Phase 17).
//!
//! Standard-first (master §23): OpenTelemetry attribute names are the Morn
//! semantic attrs; CloudEvents is the external/cross-component event
//! envelope. No isolated tracing system is invented here. Logs carry ids and
//! status, never secrets or full sensitive payloads.

use serde::{Deserialize, Serialize};

/// Morn OTel semantic attribute keys (master §23.2 / Goal §21).
pub const ATTR_WORK_ID: &str = "morn.work.id";
pub const ATTR_WORK_TYPE: &str = "morn.work.type";
pub const ATTR_PROFILE_ID: &str = "morn.profile.id";
pub const ATTR_BINDING_ID: &str = "morn.binding.id";
pub const ATTR_ATTEMPT_ID: &str = "morn.attempt.id";
pub const ATTR_CAPABILITY_ID: &str = "morn.capability.id";
pub const ATTR_CAPABILITY_DIGEST: &str = "morn.capability.digest";
pub const ATTR_PROVIDER_ID: &str = "morn.provider.id";
pub const ATTR_AUTHORITY_DECISION_ID: &str = "morn.authority.decision_id";
pub const ATTR_SITE_ID: &str = "morn.site.id";
pub const ATTR_OUTCOME_ID: &str = "morn.outcome.id";

/// All Morn semantic attribute keys, in canonical order (the trace span
/// attribute set — a conformance surface).
pub const ALL_ATTR_KEYS: &[&str] = &[
    ATTR_WORK_ID,
    ATTR_WORK_TYPE,
    ATTR_PROFILE_ID,
    ATTR_BINDING_ID,
    ATTR_ATTEMPT_ID,
    ATTR_CAPABILITY_ID,
    ATTR_CAPABILITY_DIGEST,
    ATTR_PROVIDER_ID,
    ATTR_AUTHORITY_DECISION_ID,
    ATTR_SITE_ID,
    ATTR_OUTCOME_ID,
];

/// A span attribute set for one Morn operation (ids + status, no payloads).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpanAttrs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_decision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_id: Option<String>,
}

impl SpanAttrs {
    /// Renders the attribute set as (key, value) pairs in OTel form — the
    /// keys are the canonical Morn semantic attrs.
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let fields = [
            (ATTR_WORK_ID, &self.work_id),
            (ATTR_WORK_TYPE, &self.work_type),
            (ATTR_PROFILE_ID, &self.profile_id),
            (ATTR_BINDING_ID, &self.binding_id),
            (ATTR_ATTEMPT_ID, &self.attempt_id),
            (ATTR_CAPABILITY_ID, &self.capability_id),
            (ATTR_CAPABILITY_DIGEST, &self.capability_digest),
            (ATTR_PROVIDER_ID, &self.provider_id),
            (ATTR_AUTHORITY_DECISION_ID, &self.authority_decision_id),
            (ATTR_SITE_ID, &self.site_id),
            (ATTR_OUTCOME_ID, &self.outcome_id),
        ];
        for (key, value) in fields {
            if let Some(v) = value {
                pairs.push((key.to_string(), v.clone()));
            }
        }
        pairs
    }
}

/// CloudEvents envelope (master §23.1): the external/cross-component event
/// format. Required fields: specversion / id / source / type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudEvent {
    pub specversion: String,
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Morn extension attributes (mornworkid / mornbindingid / ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mornworkid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mornbindingid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mornattemptid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mornprofile: Option<String>,
}

impl CloudEvent {
    /// Builds a valid CloudEvents v1.0 envelope for a Morn event.
    pub fn new(
        id: &str,
        source: &str,
        type_: &str,
        subject: Option<&str>,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            specversion: "1.0".to_string(),
            id: id.to_string(),
            source: source.to_string(),
            type_: type_.to_string(),
            subject: subject.map(|s| s.to_string()),
            time: Some(crate::work::now_iso()),
            data,
            mornworkid: None,
            mornbindingid: None,
            mornattemptid: None,
            mornprofile: None,
        }
    }

    /// Validates the CloudEvents required fields (specversion / id / source /
    /// type must be non-empty).
    pub fn validate(&self) -> Result<(), String> {
        if self.specversion.is_empty() {
            return Err("specversion is required".to_string());
        }
        if self.id.is_empty() {
            return Err("id is required".to_string());
        }
        if self.source.is_empty() {
            return Err("source is required".to_string());
        }
        if self.type_.is_empty() {
            return Err("type is required".to_string());
        }
        Ok(())
    }

    /// Attaches Morn extension attributes from a SpanAttrs set.
    pub fn with_span_attrs(mut self, attrs: &SpanAttrs) -> Self {
        self.mornworkid = attrs.work_id.clone();
        self.mornbindingid = attrs.binding_id.clone();
        self.mornattemptid = attrs.attempt_id.clone();
        self.mornprofile = attrs.profile_id.clone();
        self
    }
}

/// Redacts secrets from a log payload: keys named like a secret are dropped,
/// values are never logged in full (Goal §12.4 / global rule 37).
pub fn redact_for_log(payload: &serde_json::Value) -> serde_json::Value {
    const SECRET_KEY_PARTS: &[&str] = &[
        "password",
        "token",
        "secret",
        "api_key",
        "apikey",
        "credential",
    ];
    match payload {
        serde_json::Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                let lower = key.to_lowercase();
                if SECRET_KEY_PARTS.iter().any(|s| lower.contains(s)) {
                    redacted.insert(
                        key.clone(),
                        serde_json::Value::String("[REDACTED]".to_string()),
                    );
                } else {
                    redacted.insert(key.clone(), redact_for_log(value));
                }
            }
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(redact_for_log).collect())
        }
        other => other.clone(),
    }
}
