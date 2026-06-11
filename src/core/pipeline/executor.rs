//! Pipeline execution logic, including topological sort and node execution.

use crate::core::pipeline::nodes::{
    Connection, PipelineContext, PipelineData, PipelineNode, PipelineNodeExecutor,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

fn default_transform(operation: &str, input: &PipelineData) -> Result<PipelineData, String> {
    match operation {
        "to_upper" => {
            let text = input.as_text()?;
            Ok(PipelineData::Text(text.to_uppercase()))
        }
        "to_lower" => {
            let text = input.as_text()?;
            Ok(PipelineData::Text(text.to_lowercase()))
        }
        "to_number" => {
            let text = input.as_text()?;
            let n = text
                .parse::<f64>()
                .map_err(|e| format!("parse error: {}", e))?;
            Ok(PipelineData::Number(n))
        }
        "to_string" => {
            let n = input.as_number()?;
            Ok(PipelineData::Text(n.to_string()))
        }
        "double" => {
            let n = input.as_number()?;
            Ok(PipelineData::Number(n * 2.0))
        }
        "append" => {
            let text = input.as_text()?;
            Ok(PipelineData::Text(text + "_appended"))
        }
        _ => Err(format!("unknown operation '{}'", operation)),
    }
}

pub struct Pipeline {
    nodes: Vec<PipelineNode>,
    connections: Vec<Connection>,
    executors: HashMap<String, Box<dyn PipelineNodeExecutor>>,
    context: PipelineContext,
}

impl Pipeline {
    /// Create a new empty Pipeline.
    pub fn new() -> Self {
        Pipeline {
            nodes: Vec::new(),
            connections: Vec::new(),
            executors: HashMap::new(),
            context: PipelineContext::new(),
        }
    }

    /// Add a node to the pipeline.
    pub fn add_node(&mut self, node: PipelineNode) {
        self.nodes.push(node);
    }

    /// Add a connection between two nodes.
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    /// Register a custom executor for a node.
    pub fn register_executor(&mut self, node_id: &str, executor: Box<dyn PipelineNodeExecutor>) {
        self.executors.insert(node_id.to_string(), executor);
    }

    /// Get a reference to all nodes.
    pub fn nodes(&self) -> &[PipelineNode] {
        &self.nodes
    }

    /// Get a reference to all connections.
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &str) -> Option<&PipelineNode> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    /// Get the output of a specific node.
    pub fn get_node_output(&self, node_id: &str) -> Option<&PipelineData> {
        self.context.node_outputs.get(node_id)
    }

    /// Collect all node outputs into a vector.
    pub fn collect_all_outputs(&self) -> Vec<&PipelineData> {
        self.context.node_outputs.values().collect()
    }

    fn get_upstream_nodes(&self, node_id: &str) -> Vec<String> {
        self.connections
            .iter()
            .filter(|c| c.to == node_id)
            .map(|c| c.from.clone())
            .collect()
    }

    #[allow(dead_code)] /* 预留：用于完整 DAG 下游遍历 */
    fn get_downstream_nodes(&self, node_id: &str) -> Vec<String> {
        self.connections
            .iter()
            .filter(|c| c.from == node_id)
            .map(|c| c.to.clone())
            .collect()
    }

    fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in &self.nodes {
            in_degree.entry(node.id().to_string()).or_insert(0);
            adj.entry(node.id().to_string()).or_default();
        }

        for conn in &self.connections {
            adj.entry(conn.from.clone())
                .or_default()
                .push(conn.to.clone());
            *in_degree.entry(conn.to.clone()).or_insert(0) += 1;
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();
        while !queue.is_empty() {
            let node_id = queue.remove(0);
            sorted.push(node_id.clone());
            if let Some(neighbors) = adj.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err("cycle detected in pipeline graph".to_string());
        }

        Ok(sorted)
    }

    /// Execute the pipeline with an optional input.
    pub fn execute(&mut self, input: Option<PipelineData>) -> Result<(), String> {
        self.context.started_at = Some(Instant::now());

        let order = self.topological_sort()?;

        if let Some(data) = input {
            self.context.data.insert("pipeline_input".to_string(), data);
        }

        for node_id in &order {
            let node = self
                .get_node(node_id)
                .ok_or_else(|| format!("node '{}' not found", node_id))?;

            let input_data = match node {
                PipelineNode::Input { .. } => self
                    .context
                    .data
                    .get("pipeline_input")
                    .cloned()
                    .unwrap_or(PipelineData::Text(String::new())),
                PipelineNode::Timer { interval_secs, .. } => {
                    std::thread::sleep(Duration::from_secs(*interval_secs));
                    PipelineData::Text(format!("timer_{}", interval_secs))
                }
                _ => {
                    let upstream = self.get_upstream_nodes(node_id);
                    if upstream.is_empty() {
                        PipelineData::Text(String::new())
                    } else {
                        let first_upstream = &upstream[0];
                        self.context
                            .node_outputs
                            .get(first_upstream)
                            .cloned()
                            .unwrap_or(PipelineData::Text(String::new()))
                    }
                }
            };

            let output = if let Some(executor) = self.executors.get(node_id) {
                executor.execute(input_data)?
            } else {
                match node {
                    PipelineNode::Input { .. } => input_data,
                    PipelineNode::Output { .. } => input_data,
                    PipelineNode::Transform { operation, .. } => {
                        default_transform(operation, &input_data)?
                    }
                    PipelineNode::Timer { .. } => PipelineData::Text("timer_triggered".to_string()),
                    PipelineNode::Start { .. } | PipelineNode::End { .. } => {
                        return Err("start/end node not implemented in simple chain".to_string())
                    }
                    PipelineNode::LLM {
                        id: _,
                        model: _,
                        prompt: _,
                        params: _,
                    } => input_data.clone(),
                    PipelineNode::Tool {
                        id: _,
                        tool_name: _,
                        args: _,
                    } => input_data.clone(),
                    PipelineNode::Code { .. }
                    | PipelineNode::Condition { .. }
                    | PipelineNode::Loop { .. }
                    | PipelineNode::Parallel { .. }
                    | PipelineNode::Merge { .. }
                    | PipelineNode::Wait { .. }
                    | PipelineNode::HumanInput { .. }
                    | PipelineNode::Email { .. }
                    | PipelineNode::Webhook { .. }
                    | PipelineNode::Log { .. }
                    | PipelineNode::SubWorkflow { .. } => {
                        return Err("node type not implemented in simple chain".to_string())
                    }
                }
            };

            self.context.node_outputs.insert(node_id.clone(), output);
        }

        Ok(())
    }

    /// Execute nodes as a simple chain and return the last node's output.
    pub fn execute_simple_chain(&mut self, input: PipelineData) -> Result<PipelineData, String> {
        self.execute(Some(input))?;

        let last_node = self.nodes.last().ok_or("no nodes in pipeline")?;
        self.context
            .node_outputs
            .get(last_node.id())
            .cloned()
            .ok_or_else(|| "no output produced".to_string())
    }

    /// Reset the pipeline context to its initial state.
    pub fn reset(&mut self) {
        self.context = PipelineContext::new();
    }

    /// Convert this pipeline into an agentless pipeline representation.
    pub fn as_agentless(&self) -> crate::core::pipeline::agentless::AgentlessPipeline {
        let steps: Vec<crate::core::pipeline::agentless::PipelineStep> = self
            .nodes
            .iter()
            .map(|node| {
                let name = node.id().to_string();
                let command = match node {
                    PipelineNode::Input { source, .. } => format!("input:{}", source),
                    PipelineNode::Transform { operation, .. } => format!("transform:{}", operation),
                    PipelineNode::Output { target, .. } => format!("output:{}", target),
                    PipelineNode::Timer { interval_secs, .. } => {
                        format!("timer:{}", interval_secs)
                    }
                    PipelineNode::Start { .. } | PipelineNode::End { .. } => {
                        "pipeline-start-end:passthrough".to_string()
                    }
                    PipelineNode::LLM { .. }
                    | PipelineNode::Tool { .. }
                    | PipelineNode::Code { .. }
                    | PipelineNode::Condition { .. }
                    | PipelineNode::Loop { .. }
                    | PipelineNode::Parallel { .. }
                    | PipelineNode::Merge { .. }
                    | PipelineNode::Wait { .. }
                    | PipelineNode::HumanInput { .. }
                    | PipelineNode::Email { .. }
                    | PipelineNode::Webhook { .. }
                    | PipelineNode::Log { .. }
                    | PipelineNode::SubWorkflow { .. } => {
                        "pipeline-node-stub:passthrough".to_string()
                    }
                };
                crate::core::pipeline::agentless::PipelineStep {
                    name,
                    command,
                    timeout_secs: 30,
                }
            })
            .collect();
        crate::core::pipeline::agentless::AgentlessPipeline::new(steps)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("nodes", &self.nodes)
            .field("connections", &self.connections)
            .field("context", &self.context.data.len())
            .finish()
    }
}
