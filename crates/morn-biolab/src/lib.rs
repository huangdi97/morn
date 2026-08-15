//! BioLab v0.1: the first real domain slice. Dataset -> Reviewed Scientific Claim.

pub mod domain;
pub mod dream_factory;
pub mod service;

pub use dream_factory::{LiteratureSource, LoopAResult, LoopCResult};
pub use service::{BioLabService, E2eResult, E2eStep};
