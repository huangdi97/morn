//! Morn runtime: governed Action Gateway.

pub mod gateway;

pub use gateway::{
    ActionGateway, ActionPreview, AuthorizedAction, ExecutionOutcome, WorldCommitter,
};
