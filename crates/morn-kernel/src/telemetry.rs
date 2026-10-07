//! OpenTelemetry-oriented semantic attribute names for Morn.
//!
//! This module defines stable attribute keys only. Exporters/SDKs stay
//! replaceable so Morn does not invent another tracing transport.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
pub const ATTR_ACCEPTANCE_ID: &str = "morn.acceptance.id";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelemetryContext {
    pub attributes: BTreeMap<String, String>,
}

impl TelemetryContext {
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn with_work(mut self, work_id: impl Into<String>, profile_id: impl Into<String>) -> Self {
        self.set(ATTR_WORK_ID, work_id);
        self.set(ATTR_PROFILE_ID, profile_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_keys_are_namespaced_and_transport_neutral() {
        let ctx = TelemetryContext::default().with_work("work-1", "morn.factory.readonly@1.0.0");
        assert_eq!(ctx.attributes[ATTR_WORK_ID], "work-1");
        assert!(ctx.attributes.keys().all(|key| key.starts_with("morn.")));
    }
}
