//! Pipeline module — provides a DAG-based data pipeline with typed nodes, connections, executors, and an agentless pipeline.

mod nodes;
mod executor;
pub mod agentless;

#[cfg(test)]
mod tests;

pub use nodes::*;
pub use executor::*;
