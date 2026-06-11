//! Knowledge-oriented memory layers: SemanticMemory and GraphMemory.

use std::collections::{HashMap, VecDeque};

use serde_json::Value;

use super::storage::{MemoryLayer, MemoryRecord};

#[derive(Debug, Clone)]
pub struct SemanticMemory {
    facts: HashMap<String, MemoryRecord>,
    relations: Vec<RelationTriple>,
}

#[derive(Debug, Clone)]
pub struct RelationTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl SemanticMemory {
    pub fn new() -> Self {
        SemanticMemory {
            facts: HashMap::new(),
            relations: Vec::new(),
        }
    }

    pub fn add_relation(&mut self, subject: &str, predicate: &str, object: &str) {
        self.relations.push(RelationTriple {
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
        });
    }

    pub fn query_relations(&self, subject: &str) -> Vec<&RelationTriple> {
        self.relations
            .iter()
            .filter(|r| r.subject == subject)
            .collect()
    }
}

impl MemoryLayer for SemanticMemory {
    fn id(&self) -> &str {
        "semantic"
    }
    fn name(&self) -> &str {
        "Semantic Memory"
    }

    fn store(&mut self, key: &str, mut record: MemoryRecord) -> Result<(), String> {
        record.metadata.insert(
            "fact_type".to_string(),
            Value::String("factual".to_string()),
        );
        self.facts.insert(key.to_string(), record);
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self.facts.get(key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.facts.remove(key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
        let before = self.facts.len();
        if self.facts.len() > 1000 {
            let to_remove = self.facts.len() - 1000;
            let keys: Vec<String> = self.facts.keys().take(to_remove).cloned().collect();
            for k in keys {
                self.facts.remove(&k);
            }
        }
        Ok(before.saturating_sub(self.facts.len()))
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .facts
            .values()
            .filter(|r| {
                r.key.to_lowercase().contains(&q)
                    || r.content
                        .as_str()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        results.truncate(limit);
        results
    }

    fn size(&self) -> usize {
        self.facts.len()
    }
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GraphMemory {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
}

impl GraphMemory {
    pub fn new() -> Self {
        GraphMemory {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: &str, label: &str) {
        self.nodes.insert(
            id.to_string(),
            GraphNode {
                id: id.to_string(),
                label: label.to_string(),
                properties: HashMap::new(),
            },
        );
    }

    pub fn add_edge(&mut self, from: &str, to: &str, relation: &str, weight: f64) {
        self.edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            relation: relation.to_string(),
            weight,
        });
    }

    pub fn get_neighbors(&self, node_id: &str) -> Vec<(&GraphNode, &GraphEdge)> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id)
            .filter_map(|e| self.nodes.get(&e.to).map(|n| (n, e)))
            .collect()
    }

    pub fn traverse(&self, start: &str, relation: &str, max_depth: usize) -> Vec<&GraphNode> {
        let mut visited = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((start.to_string(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }
            if visited.iter().any(|n: &&GraphNode| n.id == current) {
                continue;
            }
            if let Some(node) = self.nodes.get(&current) {
                visited.push(node);
                for edge in self
                    .edges
                    .iter()
                    .filter(|e| e.from == current && e.relation == relation)
                {
                    queue.push_back((edge.to.clone(), depth + 1));
                }
            }
        }

        visited
    }
}

impl MemoryLayer for GraphMemory {
    fn id(&self) -> &str {
        "graph"
    }
    fn name(&self) -> &str {
        "Graph Memory"
    }

    fn store(&mut self, key: &str, record: MemoryRecord) -> Result<(), String> {
        self.add_node(key, record.content.as_str().unwrap_or("unknown"));
        if let Some(rels) = record.metadata.get("relations") {
            if let Some(arr) = rels.as_array() {
                for rel in arr {
                    if let (Some(from), Some(to), Some(label)) = (
                        rel.get("from").and_then(|v| v.as_str()),
                        rel.get("to").and_then(|v| v.as_str()),
                        rel.get("relation").and_then(|v| v.as_str()),
                    ) {
                        self.add_edge(from, to, label, 1.0);
                    }
                }
            }
        }
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self.nodes.get(key).map(|n| {
            MemoryRecord::new(&n.id, Value::String(n.label.clone())).with_metadata(
                "properties",
                Value::Object(
                    n.properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                ),
            )
        }))
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.nodes.remove(key);
        self.edges.retain(|e| e.from != key && e.to != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
        Ok(0)
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .nodes
            .values()
            .filter(|n| n.id.to_lowercase().contains(&q) || n.label.to_lowercase().contains(&q))
            .map(|n| MemoryRecord::new(&n.id, Value::String(n.label.clone())))
            .collect();
        results.truncate(limit);
        results
    }

    fn size(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for GraphMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_semantic_memory_store_recall() {
        let mut sm = SemanticMemory::default();
        let record = MemoryRecord::new("earth", Value::String("planet".into()));
        sm.store("earth", record).unwrap();
        let recalled = sm.recall("earth").unwrap().unwrap();
        assert_eq!(recalled.content, "planet");
        assert_eq!(recalled.metadata.get("fact_type").unwrap(), "factual");
    }

    #[test]
    fn test_semantic_memory_recall_missing() {
        let sm = SemanticMemory::default();
        assert!(sm.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_semantic_memory_forget() {
        let mut sm = SemanticMemory::default();
        sm.store("x", MemoryRecord::new("x", Value::String("y".into()))).unwrap();
        sm.forget("x").unwrap();
        assert!(sm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_semantic_memory_search() {
        let mut sm = SemanticMemory::default();
        sm.store("python", MemoryRecord::new("python", Value::String("programming language".into()))).unwrap();
        let results = sm.search("python", 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_semantic_memory_add_and_query_relations() {
        let mut sm = SemanticMemory::default();
        sm.add_relation("earth", "orbits", "sun");
        sm.add_relation("earth", "has_satellite", "moon");
        let rels = sm.query_relations("earth");
        assert_eq!(rels.len(), 2);
        let rels = sm.query_relations("sun");
        assert!(rels.is_empty());
    }

    #[test]
    fn test_semantic_memory_compress() {
        let mut sm = SemanticMemory::default();
        for i in 0..1001 {
            sm.store(&format!("k{}", i), MemoryRecord::new(&format!("k{}", i), Value::Number(i.into()))).unwrap();
        }
        let removed = sm.compress().unwrap();
        assert!(removed > 0);
        assert!(sm.size() <= 1000);
    }

    #[test]
    fn test_semantic_memory_size() {
        let mut sm = SemanticMemory::default();
        assert_eq!(sm.size(), 0);
        sm.store("a", MemoryRecord::new("a", Value::String("1".into()))).unwrap();
        assert_eq!(sm.size(), 1);
    }

    #[test]
    fn test_graph_memory_add_node() {
        let mut gm = GraphMemory::default();
        gm.add_node("n1", "Node One");
        let recalled = gm.recall("n1").unwrap().unwrap();
        assert_eq!(recalled.content, "Node One");
    }

    #[test]
    fn test_graph_memory_recall_missing() {
        let gm = GraphMemory::default();
        assert!(gm.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_graph_memory_add_edge_and_get_neighbors() {
        let mut gm = GraphMemory::default();
        gm.add_node("a", "A");
        gm.add_node("b", "B");
        gm.add_node("c", "C");
        gm.add_edge("a", "b", "knows", 1.0);
        gm.add_edge("a", "c", "knows", 1.0);
        let neighbors = gm.get_neighbors("a");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_graph_memory_traverse() {
        let mut gm = GraphMemory::default();
        gm.add_node("a", "A");
        gm.add_node("b", "B");
        gm.add_node("c", "C");
        gm.add_edge("a", "b", "knows", 1.0);
        gm.add_edge("b", "c", "knows", 1.0);
        let nodes = gm.traverse("a", "knows", 2);
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn test_graph_memory_traverse_with_depth_limit() {
        let mut gm = GraphMemory::default();
        gm.add_node("a", "A");
        gm.add_node("b", "B");
        gm.add_node("c", "C");
        gm.add_edge("a", "b", "knows", 1.0);
        gm.add_edge("b", "c", "knows", 1.0);
        let nodes = gm.traverse("a", "knows", 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_graph_memory_forget_removes_edges() {
        let mut gm = GraphMemory::default();
        gm.add_node("a", "A");
        gm.add_node("b", "B");
        gm.add_edge("a", "b", "connects", 1.0);
        gm.forget("a").unwrap();
        assert!(gm.recall("a").unwrap().is_none());
        assert_eq!(gm.get_neighbors("a").len(), 0);
    }

    #[test]
    fn test_graph_memory_search() {
        let mut gm = GraphMemory::default();
        gm.add_node("target", "Target Node");
        let results = gm.search("target", 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_graph_memory_compress() {
        let mut gm = GraphMemory::default();
        assert_eq!(gm.compress().unwrap(), 0);
    }

    #[test]
    fn test_graph_memory_size() {
        let mut gm = GraphMemory::default();
        assert_eq!(gm.size(), 0);
        gm.add_node("x", "X");
        assert_eq!(gm.size(), 1);
    }

    #[test]
    fn test_graph_memory_duplicate_node_overwrites() {
        let mut gm = GraphMemory::default();
        gm.add_node("id1", "First");
        gm.add_node("id1", "Second");
        let recalled = gm.recall("id1").unwrap().unwrap();
        assert_eq!(recalled.content, "Second");
    }
}
