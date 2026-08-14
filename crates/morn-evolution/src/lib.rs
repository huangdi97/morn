//! Governed Evolution Engine v0.1: candidate -> branch -> evaluation -> promotion.

pub mod branch;
pub mod candidate;
pub mod engine;
pub mod evaluation;
pub mod promotion;

pub use engine::EvolutionEngine;
