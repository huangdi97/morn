//! CloudEvents-compatible envelope for Morn integration events.
//!
//! Morn owns semantic event data, but not a proprietary transport envelope.
//! This minimal structure follows the CloudEvents 1.0 core attribute names so
//! event buses/connectors can map without inventing another wire format.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::event_semantics::EventSemanticDescriptor;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub specversion: String,
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataschema: Option<String>,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, String>,
    pub data: Value,
}

impl EventEnvelope {
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        event_type: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            specversion: "1.0".to_string(),
            id: id.into(),
            source: source.into(),
            event_type: event_type.into(),
            subject: None,
            time: None,
            dataschema: None,
            extensions: BTreeMap::new(),
            data,
        }
    }

    pub fn with_semantics(
        mut self,
        descriptor: &EventSemanticDescriptor,
    ) -> Result<Self, String> {
        descriptor.validate()?;
        self.extensions.insert(
            "morneventclass".to_string(),
            descriptor.class.key().to_string(),
        );
        if let Some(subject_ref) = &descriptor.subject_ref {
            self.extensions
                .insert("mornsubjectref".to_string(), subject_ref.clone());
        }
        if let Some(source_of_truth_ref) = &descriptor.source_of_truth_ref {
            self.extensions.insert(
                "mornsourcetruth".to_string(),
                source_of_truth_ref.clone(),
            );
        }
        if let Some(schema_ref) = &descriptor.schema_ref {
            self.dataschema = Some(schema_ref.clone());
        }
        Ok(self)
    }

    pub fn with_morn_context(
        mut self,
        work_id: Option<&str>,
        attempt_id: Option<&str>,
        binding_id: Option<&str>,
        profile_ref: Option<&str>,
        trace_id: Option<&str>,
    ) -> Self {
        for (key, value) in [
            ("mornworkid", work_id),
            ("mornattemptid", attempt_id),
            ("mornbindingid", binding_id),
            ("mornprofile", profile_ref),
            ("traceid", trace_id),
        ] {
            if let Some(value) = value {
                self.extensions.insert(key.to_string(), value.to_string());
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn semantic_extensions_distinguish_runtime_signal_from_external_fact() {
        let external = EventEnvelope::new(
            "evt-2",
            "morn://plant-a/cmms",
            "io.morn.action.receipt.v1",
            json!({"external_ref":"MO-88273"}),
        )
        .with_semantics(&EventSemanticDescriptor {
            class: crate::event_semantics::EventSemanticClass::ExternalObservation,
            subject_ref: Some("work://1042".to_string()),
            source_of_truth_ref: Some("cmms://plant-a".to_string()),
            schema_ref: Some("morn://schemas/action-receipt/v1".to_string()),
        })
        .unwrap();

        let value = serde_json::to_value(external).unwrap();
        assert_eq!(value["morneventclass"], "external-observation");
        assert_eq!(value["mornsourcetruth"], "cmms://plant-a");
        assert_eq!(value["dataschema"], "morn://schemas/action-receipt/v1");
    }

    #[test]
    fn envelope_uses_cloudevents_core_attribute_names() {
        let event = EventEnvelope::new(
            "evt-1",
            "morn://plant-a/cmms",
            "io.morn.action.receipt.v1",
            json!({"external_ref":"MO-88273"}),
        )
        .with_morn_context(
            Some("work-1042"),
            Some("attempt-83"),
            None,
            Some("morn.factory.readonly@1.0.0"),
            Some("trace-1"),
        );
        let value = serde_json::to_value(event).unwrap();
        assert_eq!(value["specversion"], "1.0");
        assert_eq!(value["type"], "io.morn.action.receipt.v1");
        assert_eq!(value["mornworkid"], "work-1042");
        assert_eq!(value["traceid"], "trace-1");
    }
}
