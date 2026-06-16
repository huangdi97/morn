//! knowledge — Knowledge editor with data source, process method, capacity, and TTL settings.

use crate::core::error::MornError;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeSource {
    pub name: String,
    pub source_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEditor {
    pub name: String,
    pub data_sources: Vec<KnowledgeSource>,
    pub process_method: String,
    pub update_strategy: String,
    pub capacity: usize,
    pub ttl_secs: Option<u64>,
}

impl KnowledgeEditor {
    pub fn new(name: &str) -> Self {
        KnowledgeEditor {
            name: name.to_string(),
            data_sources: Vec::new(),
            process_method: "embedding".to_string(),
            update_strategy: "manual".to_string(),
            capacity: 10_000,
            ttl_secs: None,
        }
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn set_ttl_secs(&mut self, ttl_secs: Option<u64>) {
        self.ttl_secs = ttl_secs;
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "knowledge",
            "name": self.name,
            "data_sources": self.data_sources,
            "process_method": self.process_method,
            "update_strategy": self.update_strategy,
            "capacity": self.capacity,
            "ttl_secs": self.ttl_secs,
        })
    }
}
