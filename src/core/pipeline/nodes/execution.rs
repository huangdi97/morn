//! Node execution types — executor trait and pipeline context.
use crate::core::error::MornError;
use crate::core::pipeline::nodes::PipelineData;
use std::collections::HashMap;
use std::time::Instant;

pub trait PipelineNodeExecutor: Send {
    fn execute(&self, input: PipelineData) -> Result<PipelineData, MornError>;
}

pub struct PipelineContext {
    pub data: HashMap<String, PipelineData>,
    pub node_outputs: HashMap<String, PipelineData>,
    pub started_at: Option<Instant>,
}

impl PipelineContext {
    pub fn new() -> Self {
        PipelineContext {
            data: HashMap::new(),
            node_outputs: HashMap::new(),
            started_at: None,
        }
    }
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self::new()
    }
}
