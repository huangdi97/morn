//! Actor model: ActorTemplate / ActorInstance / ActorOrigin / RepresentationContract.

pub mod actor;
pub mod representation;

pub use actor::{ActorInstance, ActorOrigin, ActorTemplate};
pub use representation::{RepresentationContract, RepresentationScope};
