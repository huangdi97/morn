//! Pipeline module — provides a DAG-based data pipeline with typed nodes, connections, executors, and an agentless pipeline.

use crate::core::error::MornError;
pub mod agentless;
mod executor;
mod nodes;
pub mod transformer;

#[cfg(test)]
mod tests;

pub use executor::*;
pub use nodes::*;

pub type PipelineTask = crate::core::workflow::WorkflowStep;
