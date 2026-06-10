//! Node execution logic: topological sort, input collection, and per-node-type execution.

use std::collections::HashMap;

use super::types::{ExecutionResult, NodeDefinition, NodeEdge, NodeGraph, NodeType};

#[derive(Debug)]
pub struct NodeExecutor;

impl NodeExecutor {
    pub fn execute(graph: &NodeGraph) -> Result<Vec<ExecutionResult>, String> {
        let node_map: HashMap<&str, &NodeDefinition> =
            graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let mut visited: HashMap<String, ExecutionResult> = HashMap::new();
        let mut execution_order = Vec::new();
        let mut sort_visited: std::collections::HashSet<String> = std::collections::HashSet::new();

        let entry_nodes: Vec<&NodeDefinition> = graph
            .nodes
            .iter()
            .filter(|n| !graph.edges.iter().any(|e| e.target == n.id))
            .collect();

        for entry in &entry_nodes {
            Self::topological_sort(
                entry.id.as_str(),
                graph,
                &node_map,
                &mut execution_order,
                &mut sort_visited,
            )?;
        }
        execution_order.reverse();

        let mut results = Vec::new();
        for node_id in execution_order {
            let node = node_map.get(node_id.as_str()).ok_or("node not found")?;
            let input = Self::collect_inputs(node, graph, &visited);
            let result = Self::execute_node(node, input);
            visited.insert(node_id.clone(), result.clone());
            results.push(result);
        }

        Ok(results)
    }

    fn topological_sort(
        start: &str,
        graph: &NodeGraph,
        node_map: &HashMap<&str, &NodeDefinition>,
        order: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if visited.contains(start) {
            return Ok(());
        }
        visited.insert(start.to_string());

        let node = node_map.get(start).ok_or("node not found")?;
        for output in &node.outputs {
            let edges: Vec<&NodeEdge> = graph
                .edges
                .iter()
                .filter(|e| e.source == node.id && e.source_output == *output)
                .collect();
            for edge in edges {
                Self::topological_sort(&edge.target, graph, node_map, order, visited)?;
            }
        }

        order.push(start.to_string());
        Ok(())
    }

    fn collect_inputs(
        node: &NodeDefinition,
        graph: &NodeGraph,
        visited: &HashMap<String, ExecutionResult>,
    ) -> HashMap<String, serde_json::Value> {
        let mut inputs = HashMap::new();
        for edge in &graph.edges {
            if edge.target == node.id {
                if let Some(result) = visited.get(&edge.source) {
                    inputs.insert(edge.target_input.clone(), result.output.clone());
                }
            }
        }
        inputs
    }

    pub(crate) fn execute_node(
        node: &NodeDefinition,
        inputs: HashMap<String, serde_json::Value>,
    ) -> ExecutionResult {
        let output = match node.node_type {
            NodeType::HttpRequest => {
                let url = node
                    .config
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let method = node
                    .config
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET");
                let mut result = serde_json::json!({
                    "url": url,
                    "method": method,
                    "status": "simulated",
                    "body": "[HttpRequest] simulated response"
                });
                if let Some(body) = inputs.get("body") {
                    result["input_body"] = body.clone();
                }
                result
            }
            NodeType::LLMCall => {
                let model = node
                    .config
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let prompt = node
                    .config
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                serde_json::json!({
                    "model": model,
                    "prompt": prompt,
                    "response": format!("[LLMCall] simulated response for: {}", prompt)
                })
            }
            NodeType::Condition => {
                let condition = node
                    .config
                    .get("condition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("true");
                let input_value = inputs.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let matched = condition == input_value || condition == "true";
                serde_json::json!({
                    "condition": condition,
                    "input": input_value,
                    "matched": matched,
                    "output": if matched { "true" } else { "false" }
                })
            }
            NodeType::Loop => {
                let max_iterations = node
                    .config
                    .get("max_iterations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5);
                let items: Vec<serde_json::Value> = inputs
                    .get("items")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let iter_count = items.len().min(max_iterations as usize);
                let results: Vec<serde_json::Value> = (0..iter_count)
                    .map(|i| {
                        serde_json::json!({
                            "iteration": i,
                            "item": items.get(i),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "total_iterations": iter_count,
                    "max_iterations": max_iterations,
                    "results": results
                })
            }
            NodeType::Transform => {
                let expression = node
                    .config
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let input_value = inputs
                    .get("data")
                    .cloned()
                    .unwrap_or(serde_json::json!(null));
                serde_json::json!({
                    "expression": expression,
                    "input": input_value,
                    "transformed": format!("[Transform] applied: {}", expression)
                })
            }
            NodeType::Merge => {
                let mut merged = serde_json::json!({});
                for (key, value) in &inputs {
                    merged[key] = value.clone();
                }
                serde_json::json!({
                    "merged": merged,
                    "source_count": inputs.len()
                })
            }
            NodeType::Split => {
                let field = node
                    .config
                    .get("field")
                    .and_then(|v| v.as_str())
                    .unwrap_or("items");
                let input_data = inputs
                    .get("data")
                    .cloned()
                    .unwrap_or(serde_json::json!(null));
                let items = match input_data.get(field) {
                    Some(arr) => arr.as_array().cloned().unwrap_or_default(),
                    None => vec![input_data],
                };
                serde_json::json!({
                    "field": field,
                    "split_count": items.len(),
                    "items": items
                })
            }
            NodeType::Code => {
                let code = node
                    .config
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let timeout = node
                    .config
                    .get("timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30);
                let sandbox = node
                    .config
                    .get("sandbox")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                serde_json::json!({
                    "code_preview": code.chars().take(50).collect::<String>(),
                    "timeout": timeout,
                    "sandbox": sandbox,
                    "input": inputs.get("input"),
                    "result": "[Code] simulated execution output"
                })
            }
            NodeType::Trigger => {
                let trigger_type = if node.config.get("cron").is_some() {
                    "cron"
                } else if node.config.get("method").is_some() {
                    "webhook"
                } else {
                    "manual"
                };
                serde_json::json!({
                    "trigger_type": trigger_type,
                    "config": node.config,
                    "triggered": true,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            NodeType::Wait => {
                let duration_secs = node
                    .config
                    .get("duration_secs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10);
                let condition = node
                    .config
                    .get("condition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                serde_json::json!({
                    "duration_secs": duration_secs,
                    "condition": condition,
                    "input": inputs.get("input"),
                    "output": "[Wait] completed",
                    "timeout": false
                })
            }
            NodeType::Switch => {
                let cases = node
                    .config
                    .get("cases")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let input_value = inputs.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let matched_case = cases
                    .iter()
                    .position(|c| c.as_str().map(|s| s == input_value).unwrap_or(false));
                let matched_case_value = matched_case
                    .map(|i| serde_json::json!(i))
                    .unwrap_or(serde_json::Value::Null);
                serde_json::json!({
                    "input": input_value,
                    "cases": cases,
                    "matched_case": matched_case_value,
                    "matched": matched_case.is_some()
                })
            }
        };

        ExecutionResult {
            node_id: node.id.clone(),
            output,
            success: true,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_executor_can_be_created() {
        let _executor = NodeExecutor;
    }
}
