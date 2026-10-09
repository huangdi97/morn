//! Dynamic Capability Fabric.
//!
//! Legacy definition/registry APIs remain available. v11.5 adds a richer
//! manifest and deterministic resolver so agents, solvers, rules, humans,
//! services and devices can be composed under the same qualification model.

pub mod definition;
pub mod discovery;
pub mod effect;
pub mod manifest;
pub mod registry;
pub mod resolver;
pub mod wasm_abi;

pub use discovery::{
    project_a2a_agent_card, project_oasf, project_xregistry, A2aAgentCardProjection,
    A2aSkillProjection, DiscoveryProvenance, ImportedDiscoveryMetadata,
    OasfDiscoveryProjection, XRegistryResourceProjection, DISCOVERY_PROJECTION_VERSION,
};
pub use definition::{
    CapabilityConsumer, CapabilityDefinition, CapabilityProvider, CapabilityRequirements,
};
pub use effect::{EffectClass, EffectContract};
pub use manifest::{
    AuthorityEnvelope, CapabilityAdmissionRef, CapabilityEconomics, CapabilityInterface,
    CapabilityManifest, CapabilityManifestId, CapabilityProvenance, CapabilityRecord,
    CapabilityStage, ExecutionRequirements, IsolationLevel,
};
pub use registry::{CapabilityBinding, CapabilityKind, CapabilityRegistry, InvocationResult};
pub use resolver::{
    CapabilityRequest, CapabilityResolver, ResolvedCapability, WorkcellMember, WorkcellPlan,
    WorkcellRequest,
};

pub use wasm_abi::{
    validate_wasm_component_binding, WasmComponentDescriptor, WasmHostGrant,
    WIT_COMPONENT_PROTOCOL,
};
