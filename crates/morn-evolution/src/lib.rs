//! Governed Evolution Engine v0.1: candidate -> branch -> evaluation -> promotion.

pub mod branch;
pub mod candidate;
pub mod distillation;
pub mod engine;
pub mod evaluation;
pub mod flywheel;
pub mod promotion;

pub use distillation::{
    qc_rule, ActorBaseline, DeterministicRule, DistillationCandidate, DistillationInput,
    DistillationOutput, DistillationService, DistilledProgram, RegressionCase, RegressionReport,
};
pub use engine::EvolutionEngine;
pub use flywheel::{
    EvolutionFlywheel, FlywheelCandidate, HumanCorrection, Pattern, PatternKind, TraceRecord,
};
