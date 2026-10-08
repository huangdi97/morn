//! Event semantic roles.
//!
//! CloudEvents is the wire envelope. Cordis and harnesses may emit rich runtime
//! events. Morn still needs a stable distinction between ephemeral software
//! lifecycle signals and durable business/control facts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EventSemanticClass {
    /// Plugin/Fiber/session/provider lifecycle signal. Useful for telemetry and
    /// health projection, but never canonical Work/Outcome truth by itself.
    RuntimeSignal,
    /// Desired-state/control request that must survive process restarts.
    ControlIntent,
    /// Durable fact produced by Morn's own semantic/control plane.
    DomainFact,
    /// Observation/receipt from an authoritative external system.
    ExternalObservation,
    /// Best-effort projection/UI notification derived from durable state.
    ProjectionNotification,
}

impl EventSemanticClass {
    pub const fn durable_required(self) -> bool {
        matches!(
            self,
            Self::ControlIntent | Self::DomainFact | Self::ExternalObservation
        )
    }

    pub const fn canonical_business_fact(self) -> bool {
        matches!(self, Self::DomainFact | Self::ExternalObservation)
    }

    pub const fn runtime_only(self) -> bool {
        matches!(self, Self::RuntimeSignal | Self::ProjectionNotification)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSemanticDescriptor {
    pub class: EventSemanticClass,
    /// Stable subject reference such as Work/Attempt/Binding/Outcome.
    pub subject_ref: Option<String>,
    /// Required for external observations so the source-of-truth is explicit.
    pub source_of_truth_ref: Option<String>,
    /// Durable semantic events should name the contract/schema that interprets
    /// their payload.
    pub schema_ref: Option<String>,
}

impl EventSemanticDescriptor {
    pub fn validate(&self) -> Result<(), String> {
        if self.class == EventSemanticClass::ExternalObservation
            && self
                .source_of_truth_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(
                "external observation requires an explicit source-of-truth reference".to_string(),
            );
        }
        if self.class.durable_required()
            && self
                .schema_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err("durable semantic event requires schema_ref".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cordis_or_harness_runtime_signal_is_not_business_truth() {
        let descriptor = EventSemanticDescriptor {
            class: EventSemanticClass::RuntimeSignal,
            subject_ref: Some("provider://dsh/session-1".to_string()),
            source_of_truth_ref: None,
            schema_ref: None,
        };
        descriptor.validate().unwrap();
        assert!(descriptor.class.runtime_only());
        assert!(!descriptor.class.canonical_business_fact());
        assert!(!descriptor.class.durable_required());
    }

    #[test]
    fn external_observation_requires_source_and_schema() {
        let invalid = EventSemanticDescriptor {
            class: EventSemanticClass::ExternalObservation,
            subject_ref: Some("work://1042".to_string()),
            source_of_truth_ref: None,
            schema_ref: Some("morn://schemas/action-receipt/v1".to_string()),
        };
        assert!(invalid.validate().is_err());

        let valid = EventSemanticDescriptor {
            class: EventSemanticClass::ExternalObservation,
            subject_ref: Some("work://1042".to_string()),
            source_of_truth_ref: Some("cmms://plant-a".to_string()),
            schema_ref: Some("morn://schemas/action-receipt/v1".to_string()),
        };
        valid.validate().unwrap();
        assert!(valid.class.durable_required());
        assert!(valid.class.canonical_business_fact());
    }

    #[test]
    fn projection_notification_is_explicitly_non_canonical() {
        let descriptor = EventSemanticDescriptor {
            class: EventSemanticClass::ProjectionNotification,
            subject_ref: Some("work://1042".to_string()),
            source_of_truth_ref: None,
            schema_ref: None,
        };
        descriptor.validate().unwrap();
        assert!(descriptor.class.runtime_only());
        assert!(!descriptor.class.canonical_business_fact());
    }
}
