//! HarnessSpec and HarnessVersion.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{HarnessSpecId, HarnessVersionId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// A versioned execution engineering contract for a digital member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSpec {
    pub id: HarnessSpecId,
    pub name: String,
    pub version: Version,
    pub runtime_profile_preferred: String,
    pub runtime_profile_fallbacks: Vec<String>,
    pub context_policy: serde_json::Value,
    pub capability_policy: serde_json::Value,
    pub orchestration: serde_json::Value,
    pub state_policy: serde_json::Value,
    pub recovery: serde_json::Value,
    pub observability: serde_json::Value,
}

impl HarnessSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: HarnessSpecId::generate_with("harness"),
            name: name.into(),
            version: Version::v1(),
            runtime_profile_preferred: "morn-native".to_string(),
            runtime_profile_fallbacks: Vec::new(),
            context_policy: serde_json::json!({}),
            capability_policy: serde_json::json!({}),
            orchestration: serde_json::json!({}),
            state_policy: serde_json::json!({"canonical_state_write": "forbidden"}),
            recovery: serde_json::json!({}),
            observability: serde_json::json!({}),
        }
    }
}

/// A specific released version of a harness spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessVersion {
    pub id: HarnessVersionId,
    pub spec_id: HarnessSpecId,
    pub version: Version,
    pub released_at: Timestamp,
}

impl HarnessVersion {
    pub fn new(spec_id: HarnessSpecId, version: Version) -> Self {
        Self {
            id: HarnessVersionId::generate_with("harnessv"),
            spec_id,
            version,
            released_at: Timestamp::now(),
        }
    }
}
