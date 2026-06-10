#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryEditor {
    pub name: String,
    pub capacity: usize,
    pub ttl_secs: Option<u64>,
    pub retrieval_method: String,
}

impl MemoryEditor {
    pub fn new(name: &str) -> Self {
        MemoryEditor {
            name: name.to_string(),
            capacity: 1000,
            ttl_secs: Some(3600),
            retrieval_method: "semantic".to_string(),
        }
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "memory",
            "name": self.name,
            "capacity": self.capacity,
            "ttl_secs": self.ttl_secs,
            "retrieval_method": self.retrieval_method,
        })
    }
}
