//! Work contracts: WorkPackage, AcceptanceSpec, OutcomeContract, ExecutionMode,
//! attention queue, durable checkpoint/recovery.

pub mod acceptance;
pub mod attention;
pub mod checkpoint;
pub mod execution_mode;
pub mod recovery;
pub mod service;
pub mod work_contract;
pub mod work_package;

pub use service::WorkService;
