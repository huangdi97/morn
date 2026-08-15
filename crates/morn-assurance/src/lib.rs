//! Morn Assurance: replay, simulation, evaluation and shadow.

pub mod certification;
pub mod evaluation;
pub mod managed_work;
pub mod replacement;
pub mod replay;
pub mod shadow;
pub mod simulation;

pub use evaluation::{EvalStep, EvaluationDecision, EvaluationResult, EvaluationRunner};
pub use replay::{ReplayReport, ReplayRunner, ReplayScenario};
pub use shadow::{ShadowComparison, ShadowRun, ShadowRunner};
pub use simulation::{FaultInjection, FaultKind, SimulationScenario};
