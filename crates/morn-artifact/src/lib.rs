//! Artifact / Decision / Outcome: evidence-first work system.

pub mod approval;
pub mod artifact;
pub mod decision;
pub mod provenance;
pub mod review;
pub mod service;
pub mod verification;

pub use artifact::{Artifact, ArtifactVersion};
pub use service::ArtifactService;
