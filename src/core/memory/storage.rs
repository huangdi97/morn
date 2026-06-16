//! Core memory storage types: MemoryRecord and the MemoryLayer trait.

use crate::core::error::MornError;
use serde_json::Value;
use std::collections::HashMap;

pub trait MemoryLayer: Send {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn store(&mut self, key: &str, data: MemoryRecord) -> Result<(), MornError>;
    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, MornError>;
    fn forget(&mut self, key: &str) -> Result<(), MornError>;
    fn compress(&mut self) -> Result<usize, MornError>;
    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord>;
    fn size(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub key: String,
    pub content: Value,
    pub metadata: HashMap<String, Value>,
    pub timestamp: String,
    pub priority: u8,
    pub ttl_secs: Option<u64>,
}

impl MemoryRecord {
    pub fn new(key: &str, content: Value) -> Self {
        MemoryRecord {
            key: key.to_string(),
            content,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            priority: 5,
            ttl_secs: None,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = Some(ttl_secs);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}
