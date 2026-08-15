//! Work contracts: WorkPackage, AcceptanceSpec, OutcomeContract, ExecutionMode,
//! attention queue, durable checkpoint/recovery.

pub mod acceptance;
pub mod attention;
pub mod checkpoint;
pub mod durable;
pub mod execution_mode;
pub mod recovery;
pub mod service;
pub mod work_contract;
pub mod work_package;
pub mod workflow;

pub use durable::DurableRuntime;
pub use service::WorkService;
pub use workflow::{RunStatus, WorkflowDefinition, WorkflowRun, WorkflowStep, WorkflowStepKind};
