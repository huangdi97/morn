//! Capability seam: Definition -> Provider -> Consumer.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{CapabilityConsumerId, CapabilityId, CapabilityProviderId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// What a capability is and what it requires/provides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    pub id: CapabilityId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub requires_capabilities: Vec<CapabilityId>,
    pub provides_effect_class: String,
}

impl CapabilityDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        requires_capabilities: Vec<CapabilityId>,
        provides_effect_class: impl Into<String>,
    ) -> Self {
        Self {
            id: CapabilityId::generate_with("cap"),
            name: name.into(),
            version: Version::v1(),
            description: description.into(),
            requires_capabilities,
            provides_effect_class: provides_effect_class.into(),
        }
    }
}

/// Declared requirements (coeffects) of a capability provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityRequirements {
    pub services: Vec<String>,
    pub world_objects: Vec<String>,
    pub data_access: Vec<String>,
    pub authority: Vec<String>,
    pub environment: Vec<String>,
}

/// A concrete provider of a capability definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProvider {
    pub id: CapabilityProviderId,
    pub definition_id: CapabilityId,
    pub provider_kind: String,
    pub status: String,
    pub created_at: Timestamp,
}

impl CapabilityProvider {
    pub fn new(definition_id: CapabilityId, provider_kind: impl Into<String>) -> Self {
        Self {
            id: CapabilityProviderId::generate_with("prov"),
            definition_id,
            provider_kind: provider_kind.into(),
            status: "unmounted".to_string(),
            created_at: Timestamp::now(),
        }
    }
}

/// A consumer that binds to a capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConsumer {
    pub id: CapabilityConsumerId,
    pub definition_id: CapabilityId,
    pub consumer_ref: String,
    pub bindings: Vec<String>,
    pub created_at: Timestamp,
}

impl CapabilityConsumer {
    pub fn new(definition_id: CapabilityId, consumer_ref: impl Into<String>) -> Self {
        Self {
            id: CapabilityConsumerId::generate_with("cons"),
            definition_id,
            consumer_ref: consumer_ref.into(),
            bindings: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}
