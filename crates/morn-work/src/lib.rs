//! Work contracts and durable work semantics.
//!
//! Existing workflow primitives remain supported. The control module adds the
//! v11.5 desired/observed Work resource used by reconciliation controllers.

pub mod acceptance;
pub mod attention;
pub mod checkpoint;
pub mod control;
pub mod durable;
pub mod execution_mode;
pub mod recovery;
pub mod service;
pub mod work_contract;
pub mod work_package;
pub mod workflow;

pub use control::{
    ConditionStatus, WorkCondition, WorkControlStatus, WorkPhase, WorkResource, WorkSpec,
};
pub use durable::DurableRuntime;
pub use service::WorkService;
pub use workflow::{RunStatus, WorkflowDefinition, WorkflowRun, WorkflowStep, WorkflowStepKind};
