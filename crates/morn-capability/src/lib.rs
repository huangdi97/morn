//! Dynamic Capability Fabric.
//!
//! Legacy definition/registry APIs remain available. v11.5 adds a richer
//! manifest and deterministic resolver so agents, solvers, rules, humans,
//! services and devices can be composed under the same qualification model.

pub mod definition;
pub mod effect;
pub mod manifest;
pub mod registry;
pub mod resolver;

pub use definition::{
    CapabilityConsumer, CapabilityDefinition, CapabilityProvider, CapabilityRequirements,
};
pub use effect::{EffectClass, EffectContract};
pub use manifest::{
    AuthorityEnvelope, CapabilityAdmissionRef, CapabilityEconomics, CapabilityInterface, CapabilityManifest,
    CapabilityManifestId, CapabilityProvenance, CapabilityRecord, CapabilityStage,
    ExecutionRequirements, IsolationLevel,
};
pub use registry::{CapabilityBinding, CapabilityKind, CapabilityRegistry, InvocationResult};
pub use resolver::{
    CapabilityRequest, CapabilityResolver, ResolvedCapability, WorkcellMember, WorkcellPlan,
    WorkcellRequest,
};
