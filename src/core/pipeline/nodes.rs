//! Node type definitions and related data types for the pipeline system.

use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineData {
    Text(String),
    Bytes(Vec<u8>),
    Number(f64),
    Table(Vec<Vec<Value>>),
    Json(Value),
}

impl PipelineData {
    pub fn as_text(&self) -> Result<String, String> {
        match self {
            PipelineData::Text(t) => Ok(t.clone()),
            PipelineData::Json(v) => Ok(v.to_string()),
            PipelineData::Number(n) => Ok(n.to_string()),
            _ => Err("cannot convert to text".to_string()),
        }
    }

    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            PipelineData::Number(n) => Ok(*n),
            PipelineData::Text(t) => t.parse::<f64>().map_err(|e| e.to_string()),
            _ => Err("cannot convert to number".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PipelineNode {
    Input {
        id: String,
        source: String,
    },
    Transform {
        id: String,
        operation: String,
        params: Value,
    },
    Output {
        id: String,
        target: String,
    },
    Timer {
        id: String,
        interval_secs: u64,
    },
}

impl PipelineNode {
    pub fn id(&self) -> &str {
        match self {
            PipelineNode::Input { id, .. } => id,
            PipelineNode::Transform { id, .. } => id,
            PipelineNode::Output { id, .. } => id,
            PipelineNode::Timer { id, .. } => id,
        }
    }

    pub fn node_type(&self) -> &str {
        match self {
            PipelineNode::Input { .. } => "input",
            PipelineNode::Transform { .. } => "transform",
            PipelineNode::Output { .. } => "output",
            PipelineNode::Timer { .. } => "timer",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub from_port: String,
    pub to_port: String,
}

pub trait PipelineNodeExecutor: Send {
    fn execute(&self, input: PipelineData) -> Result<PipelineData, String>;
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