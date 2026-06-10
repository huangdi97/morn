//! Node system: types, executor, and registry. This module re-exports all public items from sibling modules.

pub use super::executor::NodeExecutor;
pub use super::registry::NodeRegistry;
pub use super::types::{ExecutionResult, NodeDefinition, NodeEdge, NodeGraph, NodeTemplate, NodeType};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_graph() -> NodeGraph {
        NodeGraph {
            nodes: vec![
                NodeDefinition {
                    id: "node_1".into(),
                    node_type: NodeType::HttpRequest,
                    label: "Fetch Data".into(),
                    config: serde_json::json!({"url": "https://api.example.com/data", "method": "GET"}),
                    inputs: vec![],
                    outputs: vec!["response".into()],
                },
                NodeDefinition {
                    id: "node_2".into(),
                    node_type: NodeType::Transform,
                    label: "Parse JSON".into(),
                    config: serde_json::json!({"expression": "json_parse"}),
                    inputs: vec!["data".into()],
                    outputs: vec!["transformed".into()],
                },
                NodeDefinition {
                    id: "node_3".into(),
                    node_type: NodeType::Condition,
                    label: "Check Status".into(),
                    config: serde_json::json!({"condition": "200", "operator": "equals"}),
                    inputs: vec!["value".into()],
                    outputs: vec!["true".into(), "false".into()],
                },
            ],
            edges: vec![
                NodeEdge {
                    id: "e1".into(),
                    source: "node_1".into(),
                    source_output: "response".into(),
                    target: "node_2".into(),
                    target_input: "data".into(),
                },
                NodeEdge {
                    id: "e2".into(),
                    source: "node_2".into(),
                    source_output: "transformed".into(),
                    target: "node_3".into(),
                    target_input: "value".into(),
                },
            ],
        }
    }

    #[test]
    fn test_execute_simple_graph() {
        let graph = sample_graph();
        let results = NodeExecutor::execute(&graph).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_execution_order_respected() {
        let graph = sample_graph();
        let results = NodeExecutor::execute(&graph).unwrap();
        assert_eq!(results[0].node_id, "node_1");
        assert_eq!(results[1].node_id, "node_2");
        assert_eq!(results[2].node_id, "node_3");
    }

    #[test]
    fn test_empty_graph() {
        let graph = NodeGraph {
            nodes: vec![],
            edges: vec![],
        };
        let results = NodeExecutor::execute(&graph).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_node_registry_has_20_templates() {
        let templates = NodeRegistry::all_templates();
        assert!(templates.len() >= 22);
        assert!(templates.iter().any(|t| t.node_type == NodeType::HttpRequest));
        assert!(templates.iter().any(|t| t.node_type == NodeType::LLMCall));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Condition));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Loop));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Transform));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Merge));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Split));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Code));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Trigger));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Wait));
        assert!(templates.iter().any(|t| t.node_type == NodeType::Switch));
    }

    #[test]
    fn test_http_request_execution() {
        let node = NodeDefinition {
            id: "test".into(),
            node_type: NodeType::HttpRequest,
            label: "Test".into(),
            config: serde_json::json!({"url": "https://example.com", "method": "POST"}),
            inputs: vec!["body".into()],
            outputs: vec!["response".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("body".into(), serde_json::json!({"key": "value"}));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["method"], "POST");
        assert_eq!(result.output["input_body"]["key"], "value");
    }

    #[test]
    fn test_condition_execution() {
        let node = NodeDefinition {
            id: "cond".into(),
            node_type: NodeType::Condition,
            label: "Test".into(),
            config: serde_json::json!({"condition": "active", "operator": "equals"}),
            inputs: vec!["value".into()],
            outputs: vec!["true".into(), "false".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("value".into(), serde_json::json!("active"));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["matched"], true);
    }

    #[test]
    fn test_loop_execution() {
        let node = NodeDefinition {
            id: "loop".into(),
            node_type: NodeType::Loop,
            label: "Test".into(),
            config: serde_json::json!({"max_iterations": 3}),
            inputs: vec!["items".into()],
            outputs: vec!["iteration".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("items".into(), serde_json::json!(["a", "b", "c", "d", "e"]));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["total_iterations"], 3);
    }

    #[test]
    fn test_merge_execution() {
        let node = NodeDefinition {
            id: "merge".into(),
            node_type: NodeType::Merge,
            label: "Test".into(),
            config: serde_json::json!({"strategy": "object_merge"}),
            inputs: vec!["source_a".into(), "source_b".into()],
            outputs: vec!["merged".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("source_a".into(), serde_json::json!({"a": 1}));
        inputs.insert("source_b".into(), serde_json::json!({"b": 2}));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["source_count"], 2);
    }

    #[test]
    fn test_code_node_execution() {
        let node = NodeDefinition {
            id: "code".into(),
            node_type: NodeType::Code,
            label: "Test".into(),
            config: serde_json::json!({"code": "print('hello')", "timeout": 10, "sandbox": true}),
            inputs: vec!["input".into()],
            outputs: vec!["result".into(), "error".into()],
        };
        let result = NodeExecutor::execute_node(&node, HashMap::new());
        assert!(result.success);
        assert!(result.output["code_preview"].as_str().unwrap_or("").contains("print"));
    }

    #[test]
    fn test_trigger_node_execution() {
        let node = NodeDefinition {
            id: "trigger".into(),
            node_type: NodeType::Trigger,
            label: "Test".into(),
            config: serde_json::json!({"cron": "0 * * * *"}),
            inputs: vec![],
            outputs: vec!["trigger".into()],
        };
        let result = NodeExecutor::execute_node(&node, HashMap::new());
        assert!(result.success);
        assert_eq!(result.output["trigger_type"], "cron");
    }

    #[test]
    fn test_wait_node_execution() {
        let node = NodeDefinition {
            id: "wait".into(),
            node_type: NodeType::Wait,
            label: "Test".into(),
            config: serde_json::json!({"duration_secs": 5}),
            inputs: vec!["input".into()],
            outputs: vec!["output".into(), "timeout".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), serde_json::json!("data"));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["duration_secs"], 5);
    }

    #[test]
    fn test_switch_node_execution() {
        let node = NodeDefinition {
            id: "switch".into(),
            node_type: NodeType::Switch,
            label: "Test".into(),
            config: serde_json::json!({"cases": ["a", "b", "c"]}),
            inputs: vec!["value".into()],
            outputs: vec!["case_0".into(), "case_1".into(), "default".into()],
        };
        let mut inputs = HashMap::new();
        inputs.insert("value".into(), serde_json::json!("b"));
        let result = NodeExecutor::execute_node(&node, inputs);
        assert!(result.success);
        assert_eq!(result.output["matched_case"], serde_json::json!(1));
    }
}