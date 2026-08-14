//! Dynamic Capability Fabric: definitions, providers, consumers and effect contracts.

pub mod definition;
pub mod effect;

pub use definition::{
    CapabilityConsumer, CapabilityDefinition, CapabilityProvider, CapabilityRequirements,
};
pub use effect::{EffectClass, EffectContract};
