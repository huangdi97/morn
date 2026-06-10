//! Vector knowledge representation with cosine-similarity search.

use std::collections::HashMap;

use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

use super::{Knowledge, KnowledgeItem};

#[derive(Debug, Clone)]
pub struct VectorKnowledge {
    id: String,
    _name: String,
    vectors: HashMap<String, Vec<f64>>,
    texts: HashMap<String, String>,
}

impl VectorKnowledge {
    pub fn new(id: &str, name: &str) -> Self {
        VectorKnowledge {
            id: id.to_string(),
            _name: name.to_string(),
            vectors: HashMap::new(),
            texts: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, key: &str, text: &str, vector: Vec<f64>) {
        self.vectors.insert(key.to_string(), vector);
        self.texts.insert(key.to_string(), text.to_string());
    }

    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

impl Component for VectorKnowledge {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "knowledge"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for VectorKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "search text".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "json".into(),
                description: "ranked results".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for VectorKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Knowledge for VectorKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        if !self.texts.contains_key(query) && !self.vectors.contains_key(query) {
            return Ok(vec![]);
        }
        let query_vec = self.vectors.get(query).cloned().unwrap_or_default();
        let mut scored: Vec<(f64, &String)> = self
            .texts
            .keys()
            .filter(|k| *k != query)
            .map(|k| {
                let vec = self.vectors.get(k).cloned().unwrap_or_default();
                let score = Self::cosine_similarity(&query_vec, &vec);
                (score, k)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored
            .into_iter()
            .take(5)
            .map(|(score, key)| KnowledgeItem {
                key: key.clone(),
                value: self.texts.get(key).cloned().unwrap_or_default(),
                source: format!("vector_similarity({:.4})", score),
            })
            .collect())
    }

    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String> {
        for item in items {
            self.texts.insert(item.key.clone(), item.value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_knowledge_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let c = vec![1.0, 1.0];
        assert!((VectorKnowledge::cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
        assert!((VectorKnowledge::cosine_similarity(&a, &c) - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_vector_knowledge_query() {
        let mut vk = VectorKnowledge::new("vec-1", "TestVectors");
        vk.add_document("doc1", "hello world", vec![1.0, 0.0, 0.0]);
        vk.add_document("doc2", "goodbye world", vec![0.0, 1.0, 0.0]);
        vk.add_document("doc3", "hello there", vec![1.0, 1.0, 0.0]);
        let results = vk.query("doc1").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_vector_knowledge_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = VectorKnowledge::cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_knowledge_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = VectorKnowledge::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_knowledge_query_returns_empty_for_no_match() {
        let vk = VectorKnowledge::new("vec-2", "EmptyVectors");
        let results = vk.query("nonexistent").unwrap();
        assert!(results.is_empty());
    }
}
