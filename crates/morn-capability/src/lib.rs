//! Dynamic Capability Fabric: definitions, providers, consumers and effect contracts.

pub mod definition;
pub mod effect;
pub mod registry;

pub use definition::{
    CapabilityConsumer, CapabilityDefinition, CapabilityProvider, CapabilityRequirements,
};
pub use effect::{EffectClass, EffectContract};
pub use registry::{CapabilityBinding, CapabilityKind, CapabilityRegistry, InvocationResult};
