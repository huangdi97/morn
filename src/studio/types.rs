//! Type definitions for the node system: node types, definitions, edges, graphs, and templates.

use crate::core::error::MornError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    HttpRequest,
    LLMCall,
    Condition,
    Loop,
    Transform,
    Merge,
    Split,
    Code,
    Trigger,
    Wait,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub config: serde_json::Value,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEdge {
    pub id: String,
    pub source: String,
    pub source_output: String,
    pub target: String,
    pub target_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<NodeEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub node_id: String,
    pub output: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTemplate {
    pub node_type: NodeType,
    pub label: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub default_config: serde_json::Value,
    pub inputs: Vec<&'static str>,
    pub outputs: Vec<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_node_type_variants() {
        let variants = vec![
            NodeType::HttpRequest,
            NodeType::LLMCall,
            NodeType::Condition,
            NodeType::Loop,
            NodeType::Transform,
            NodeType::Merge,
            NodeType::Split,
            NodeType::Code,
            NodeType::Trigger,
            NodeType::Wait,
            NodeType::Switch,
        ];
        assert_eq!(variants.len(), 11);
    }
}
