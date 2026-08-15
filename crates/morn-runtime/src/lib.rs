//! Morn runtime: governed Action Gateway + RuntimeProvider protocol.

pub mod gateway;
pub mod runtime_provider;

pub use gateway::{
    ActionGateway, ActionPreview, AuthorizedAction, ExecutionOutcome, WorldCommitter,
};
pub use runtime_provider::{run_runtime_conformance, FixtureRuntime, RuntimeProvider};
