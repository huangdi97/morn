//! HarnessBinding and RuntimeBinding are separated: an actor can switch
//! harness/runtime while keeping identity, workspace and work unchanged.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ActorInstanceId, HarnessBindingId, HarnessSpecId, RuntimeBindingId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// Which harness spec/version an actor is bound to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessBinding {
    pub id: HarnessBindingId,
    pub actor_id: ActorInstanceId,
    pub spec_id: HarnessSpecId,
    pub spec_version: Version,
    pub runtime_profile_policy: String,
    pub created_at: Timestamp,
}

impl HarnessBinding {
    pub fn new(
        actor_id: ActorInstanceId,
        spec_id: HarnessSpecId,
        spec_version: Version,
    ) -> Self {
        Self {
            id: HarnessBindingId::generate_with("hb"),
            actor_id,
            spec_id,
            spec_version,
            runtime_profile_policy: "preferred".to_string(),
            created_at: Timestamp::now(),
        }
    }
}

/// Where and on what the harness actually runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub id: RuntimeBindingId,
    pub actor_id: ActorInstanceId,
    pub provider: String,
    pub endpoint: Option<String>,
    pub session_ref: Option<String>,
    pub created_at: Timestamp,
}

impl RuntimeBinding {
    pub fn new(actor_id: ActorInstanceId, provider: impl Into<String>) -> Self {
        Self {
            id: RuntimeBindingId::generate_with("rb"),
            actor_id,
            provider: provider.into(),
            endpoint: None,
            session_ref: None,
            created_at: Timestamp::now(),
        }
    }
}
