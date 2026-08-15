//! WorkGraph: a graph of WorkNodes (work, not agents) and typed edges.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ProblemSpecId, WorkEdgeId, WorkGraphId, WorkNodeId};

/// Nature of work in a node (mirrors ExecutionMode.WorkNature for planning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkNature {
    Deterministic,
    Probabilistic,
    Physical,
    Regulated,
    Social,
    Mixed,
}

/// A WorkNode is a unit of WORK, not an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkNode {
    pub id: WorkNodeId,
    pub name: String,
    pub objective: String,
    pub nature: WorkNature,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub acceptance: Vec<String>,
    pub risks: Vec<String>,
}

impl WorkNode {
    pub fn new(name: impl Into<String>, objective: impl Into<String>, nature: WorkNature) -> Self {
        Self {
            id: WorkNodeId::generate_with("wnode"),
            name: name.into(),
            objective: objective.into(),
            nature,
            inputs: Vec::new(),
            outputs: Vec::new(),
            acceptance: Vec::new(),
            risks: Vec::new(),
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<String>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn with_acceptance(mut self, acceptance: Vec<String>) -> Self {
        self.acceptance = acceptance;
        self
    }
}

/// Kind of dependency between two work nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkEdgeKind {
    Data,
    Approval,
    State,
    Temporal,
    Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkEdge {
    pub id: WorkEdgeId,
    pub from: WorkNodeId,
    pub to: WorkNodeId,
    pub kind: WorkEdgeKind,
    pub label: String,
}

impl WorkEdge {
    pub fn new(
        from: WorkNodeId,
        to: WorkNodeId,
        kind: WorkEdgeKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: WorkEdgeId::generate_with("wedge"),
            from,
            to,
            kind,
            label: label.into(),
        }
    }
}

/// Errors when validating a work graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkGraphError {
    pub message: String,
}

/// A directed graph of work nodes with typed edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkGraph {
    pub id: WorkGraphId,
    pub problem_spec_id: ProblemSpecId,
    pub nodes: Vec<WorkNode>,
    pub edges: Vec<WorkEdge>,
}

impl WorkGraph {
    pub fn new(problem_spec_id: ProblemSpecId) -> Self {
        Self {
            id: WorkGraphId::generate_with("wgraph"),
            problem_spec_id,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: WorkNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: WorkEdge) {
        self.edges.push(edge);
    }

    pub fn node(&self, id: &WorkNodeId) -> Option<&WorkNode> {
        self.nodes.iter().find(|n| n.id == *id)
    }

    /// Validate: no cycle, every edge references existing nodes, every node has acceptance.
    pub fn validate(&self) -> Result<()> {
        for edge in &self.edges {
            if self.node(&edge.from).is_none() || self.node(&edge.to).is_none() {
                return Err(Error::validation(format!(
                    "edge {} references a missing node",
                    edge.id
                )));
            }
        }
        for node in &self.nodes {
            if node.acceptance.is_empty() {
                return Err(Error::validation(format!(
                    "work node {} has no acceptance criteria",
                    node.name
                )));
            }
        }
        if self.has_cycle() {
            return Err(Error::validation("work graph contains a cycle"));
        }
        Ok(())
    }

    /// Detect a cycle with DFS (returns true if a cycle exists).
    pub fn has_cycle(&self) -> bool {
        fn dfs(
            node: &WorkNodeId,
            graph: &WorkGraph,
            visiting: &mut std::collections::HashSet<WorkNodeId>,
            visited: &mut std::collections::HashSet<WorkNodeId>,
        ) -> bool {
            if visiting.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }
            visiting.insert(node.clone());
            for edge in graph.edges.iter().filter(|e| &e.from == node) {
                if dfs(&edge.to, graph, visiting, visited) {
                    return true;
                }
            }
            visiting.remove(node);
            visited.insert(node.clone());
            false
        }

        let mut visiting = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        for node in &self.nodes {
            if dfs(&node.id, self, &mut visiting, &mut visited) {
                return true;
            }
        }
        false
    }

    /// Nodes in topological order (None if cyclic).
    pub fn topological_order(&self) -> Option<Vec<WorkNodeId>> {
        if self.has_cycle() {
            return None;
        }
        let mut in_degree: std::collections::HashMap<WorkNodeId, usize> =
            self.nodes.iter().map(|n| (n.id.clone(), 0usize)).collect();
        for edge in &self.edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }
        let mut queue: Vec<WorkNodeId> = in_degree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(id, _)| id.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(id) = queue.pop() {
            order.push(id.clone());
            for edge in self.edges.iter().filter(|e| e.from == id) {
                let d = in_degree.get_mut(&edge.to).unwrap();
                *d -= 1;
                if *d == 0 {
                    queue.push(edge.to.clone());
                }
            }
        }
        if order.len() == self.nodes.len() {
            Some(order)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_graph_validates_and_detects_cycles() {
        let ws = ProblemSpecId::generate();
        let mut graph = WorkGraph::new(ws);
        let a = WorkNode::new("collect", "collect data", WorkNature::Deterministic)
            .with_acceptance(vec!["data locked".into()]);
        let b = WorkNode::new("analyze", "analyze", WorkNature::Probabilistic)
            .with_acceptance(vec!["report".into()]);
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(WorkEdge::new(
            a_id.clone(),
            b_id.clone(),
            WorkEdgeKind::Data,
            "input",
        ));
        assert!(graph.validate().is_ok());
        // add cycle b -> a
        graph.add_edge(WorkEdge::new(b_id, a_id, WorkEdgeKind::State, "feedback"));
        assert!(graph.validate().is_err());
        assert!(graph.has_cycle());
        assert!(graph.topological_order().is_none());
    }

    #[test]
    fn node_without_acceptance_fails_validation() {
        let mut graph = WorkGraph::new(ProblemSpecId::generate());
        graph.add_node(WorkNode::new(
            "x",
            "no acceptance",
            WorkNature::Deterministic,
        ));
        assert!(graph.validate().is_err());
    }

    #[test]
    fn topological_order_produces_valid_sequence() {
        let mut graph = WorkGraph::new(ProblemSpecId::generate());
        let a = WorkNode::new("a", "first", WorkNature::Deterministic)
            .with_acceptance(vec!["x".into()]);
        let b = WorkNode::new("b", "second", WorkNature::Deterministic)
            .with_acceptance(vec!["y".into()]);
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(WorkEdge::new(a_id, b_id, WorkEdgeKind::Temporal, "after"));
        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], graph.nodes[0].id);
    }
}
