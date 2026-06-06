use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static RNG_STATE: AtomicU64 = AtomicU64::new(42);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeKind {
    Agent,
    Task,
    DataFlow,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    pub metadata: HashMap<String, Value>,
    pub position: Option<Position>,
    pub velocity: Option<Velocity>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub weight: f64,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForceGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct Visualization3D {
    pub graph: ForceGraphData,
}

impl Default for Visualization3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Visualization3D {
    pub fn new() -> Self {
        Visualization3D {
            graph: ForceGraphData {
                nodes: Vec::new(),
                edges: Vec::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    pub fn add_node(
        &mut self,
        kind: NodeKind,
        label: &str,
        metadata: HashMap<String, Value>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.graph.nodes.push(GraphNode {
            id: id.clone(),
            kind,
            label: label.to_string(),
            metadata,
            position: None,
            velocity: None,
        });
        id
    }

    pub fn add_edge(
        &mut self,
        source: &str,
        target: &str,
        label: &str,
        weight: f64,
        metadata: HashMap<String, Value>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.graph.edges.push(GraphEdge {
            id: id.clone(),
            source: source.to_string(),
            target: target.to_string(),
            label: label.to_string(),
            weight,
            metadata,
        });
        id
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.graph.nodes.retain(|n| n.id != node_id);
        self.graph
            .edges
            .retain(|e| e.source != node_id && e.target != node_id);
    }

    pub fn remove_edge(&mut self, edge_id: &str) {
        self.graph.edges.retain(|e| e.id != edge_id);
    }

    pub fn get_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.graph.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn get_edge(&self, edge_id: &str) -> Option<&GraphEdge> {
        self.graph.edges.iter().find(|e| e.id == edge_id)
    }

    pub fn find_neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        let neighbor_ids: Vec<&str> = self
            .graph
            .edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .map(|e| {
                if e.source == node_id {
                    e.target.as_str()
                } else {
                    e.source.as_str()
                }
            })
            .collect();
        self.graph
            .nodes
            .iter()
            .filter(|n| neighbor_ids.contains(&n.id.as_str()))
            .collect()
    }

    pub fn apply_force(&mut self, repulsion: f64, attraction: f64) {
        let n = self.graph.nodes.len();
        if n == 0 {
            return;
        }
        let mut forces: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, 0.0); n];
        let mut positions: Vec<(f64, f64, f64)> = Vec::with_capacity(n);

        for node in &self.graph.nodes {
            let p = node.position.clone().unwrap_or(Position {
                x: rand_pos(),
                y: rand_pos(),
                z: rand_pos(),
            });
            positions.push((p.x, p.y, p.z));
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let (ax, ay, az) = positions[i];
                let (bx, by, bz) = positions[j];
                let dx = ax - bx;
                let dy = ay - by;
                let dz = az - bz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                let force = repulsion / (dist * dist);
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                let fz = force * dz / dist;
                forces[i].0 += fx;
                forces[i].1 += fy;
                forces[i].2 += fz;
                forces[j].0 -= fx;
                forces[j].1 -= fy;
                forces[j].2 -= fz;
            }
        }

        for edge in &self.graph.edges {
            let si = self.graph.nodes.iter().position(|n| n.id == edge.source);
            let ti = self.graph.nodes.iter().position(|n| n.id == edge.target);
            if let (Some(si), Some(ti)) = (si, ti) {
                let (sx, sy, sz) = positions[si];
                let (tx, ty, tz) = positions[ti];
                let dx = tx - sx;
                let dy = ty - sy;
                let dz = tz - sz;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                let force = attraction * edge.weight * dist;
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                let fz = force * dz / dist;
                forces[si].0 += fx;
                forces[si].1 += fy;
                forces[si].2 += fz;
                forces[ti].0 -= fx;
                forces[ti].1 -= fy;
                forces[ti].2 -= fz;
            }
        }

        for (i, node) in self.graph.nodes.iter_mut().enumerate() {
            let pos = node.position.get_or_insert_with(|| Position {
                x: rand_pos(),
                y: rand_pos(),
                z: rand_pos(),
            });
            let vel = node.velocity.get_or_insert(Velocity {
                vx: 0.0,
                vy: 0.0,
                vz: 0.0,
            });
            vel.vx = (vel.vx + forces[i].0) * 0.85;
            vel.vy = (vel.vy + forces[i].1) * 0.85;
            vel.vz = (vel.vz + forces[i].2) * 0.85;
            pos.x += vel.vx;
            pos.y += vel.vy;
            pos.z += vel.vz;
        }

        self.graph.timestamp = chrono::Utc::now().to_rfc3339();
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.graph).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str::<ForceGraphData>(json)
            .ok()
            .map(|graph| Visualization3D { graph })
    }

    pub fn clear(&mut self) {
        self.graph.nodes.clear();
        self.graph.edges.clear();
        self.graph.timestamp = chrono::Utc::now().to_rfc3339();
    }
}

fn rand_pos() -> f64 {
    let s = RNG_STATE.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed);
    let x = s.wrapping_mul(0xbf58476d1ce4e5b9);
    let y = (x ^ (x >> 30)).wrapping_mul(0x94d049bb133111eb);
    let z = y ^ (y >> 27);
    (z as f64 / u64::MAX as f64) * 200.0 - 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_viz() -> Visualization3D {
        Visualization3D::new()
    }

    fn node_ids(viz: &mut Visualization3D, n: usize) -> Vec<String> {
        (0..n)
            .map(|i| viz.add_node(NodeKind::Agent, &format!("node{}", i), HashMap::new()))
            .collect()
    }

    #[test]
    fn test_new() {
        let viz = make_viz();
        assert!(viz.graph.nodes.is_empty());
        assert!(viz.graph.edges.is_empty());
        assert!(!viz.graph.timestamp.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut viz = make_viz();
        let id = viz.add_node(NodeKind::Agent, "test-agent", HashMap::new());
        assert_eq!(viz.graph.nodes.len(), 1);
        let node = &viz.graph.nodes[0];
        assert_eq!(node.id, id);
        assert_eq!(node.label, "test-agent");
        assert!(matches!(node.kind, NodeKind::Agent));
        assert!(node.position.is_none());
        assert!(node.velocity.is_none());
    }

    #[test]
    fn test_add_node_kinds() {
        let mut viz = make_viz();
        viz.add_node(NodeKind::Agent, "a", HashMap::new());
        viz.add_node(NodeKind::Task, "b", HashMap::new());
        viz.add_node(NodeKind::DataFlow, "c", HashMap::new());
        assert_eq!(viz.graph.nodes.len(), 3);
    }

    #[test]
    fn test_add_edge() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 2);
        let edge_id = viz.add_edge(&ids[0], &ids[1], "connects", 1.0, HashMap::new());
        assert_eq!(viz.graph.edges.len(), 1);
        let edge = &viz.graph.edges[0];
        assert_eq!(edge.id, edge_id);
        assert_eq!(edge.source, ids[0]);
        assert_eq!(edge.target, ids[1]);
        assert_eq!(edge.label, "connects");
        assert_eq!(edge.weight, 1.0);
    }

    #[test]
    fn test_remove_node() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 3);
        viz.add_edge(&ids[0], &ids[1], "e1", 1.0, HashMap::new());
        viz.add_edge(&ids[1], &ids[2], "e2", 1.0, HashMap::new());
        viz.remove_node(&ids[1]);
        assert_eq!(viz.graph.nodes.len(), 2);
        assert_eq!(viz.graph.edges.len(), 0);
    }

    #[test]
    fn test_remove_edge() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 2);
        let eid = viz.add_edge(&ids[0], &ids[1], "e", 1.0, HashMap::new());
        viz.remove_edge(&eid);
        assert_eq!(viz.graph.edges.len(), 0);
    }

    #[test]
    fn test_get_node() {
        let mut viz = make_viz();
        let id = viz.add_node(NodeKind::Task, "find", HashMap::new());
        let node = viz.get_node(&id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().label, "find");
        assert!(viz.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_get_edge() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 2);
        let eid = viz.add_edge(&ids[0], &ids[1], "e", 1.0, HashMap::new());
        assert!(viz.get_edge(&eid).is_some());
        assert!(viz.get_edge("nonexistent").is_none());
    }

    #[test]
    fn test_find_neighbors() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 4);
        viz.add_edge(&ids[0], &ids[1], "", 1.0, HashMap::new());
        viz.add_edge(&ids[0], &ids[2], "", 1.0, HashMap::new());
        viz.add_edge(&ids[3], &ids[1], "", 1.0, HashMap::new());
        let neighbors = viz.find_neighbors(&ids[0]);
        assert_eq!(neighbors.len(), 2);
        let nlabels: Vec<&str> = neighbors.iter().map(|n| n.label.as_str()).collect();
        assert!(nlabels.contains(&"node1"));
        assert!(nlabels.contains(&"node2"));
    }

    #[test]
    fn test_find_neighbors_none() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 1);
        let neighbors = viz.find_neighbors(&ids[0]);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn test_apply_force_empty() {
        let mut viz = make_viz();
        viz.apply_force(100.0, 0.01);
        assert!(viz.graph.nodes.is_empty());
    }

    #[test]
    fn test_apply_force_positions_assigned() {
        let mut viz = make_viz();
        let ids = node_ids(&mut viz, 3);
        viz.add_edge(&ids[0], &ids[1], "", 1.0, HashMap::new());
        viz.apply_force(100.0, 0.01);
        for node in &viz.graph.nodes {
            assert!(node.position.is_some());
            assert!(node.velocity.is_some());
        }
    }

    #[test]
    fn test_to_json_roundtrip() {
        let mut viz = make_viz();
        node_ids(&mut viz, 2);
        let json = viz.to_json();
        let parsed = Visualization3D::from_json(&json);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().graph.nodes.len(), 2);
    }

    #[test]
    fn test_from_json_invalid() {
        let result = Visualization3D::from_json("not json");
        assert!(result.is_none());
    }

    #[test]
    fn test_clear() {
        let mut viz = make_viz();
        node_ids(&mut viz, 5);
        viz.clear();
        assert!(viz.graph.nodes.is_empty());
        assert!(viz.graph.edges.is_empty());
        assert!(!viz.graph.timestamp.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_graph_data() {
        let mut viz = make_viz();
        let id = viz.add_node(NodeKind::DataFlow, "data", HashMap::new());
        viz.add_edge(&id, &id, "self", 0.5, HashMap::new());
        let json = viz.to_json();
        let restored = Visualization3D::from_json(&json).unwrap();
        assert_eq!(restored.graph.nodes.len(), 1);
        assert_eq!(restored.graph.edges.len(), 1);
        assert_eq!(restored.graph.nodes[0].label, "data");
        assert_eq!(restored.graph.edges[0].weight, 0.5);
    }
}
