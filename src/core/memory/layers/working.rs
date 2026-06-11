//! Working memory — short-term task-context storage.
use std::collections::{HashMap, VecDeque};

use super::super::storage::{MemoryLayer, MemoryRecord};

#[derive(Debug, Clone)]
pub struct WorkingMemory {
    data: HashMap<String, MemoryRecord>,
    max_capacity: usize,
    context_stack: VecDeque<String>,
}

impl WorkingMemory {
    pub fn new(max_capacity: usize) -> Self {
        WorkingMemory {
            data: HashMap::new(),
            max_capacity,
            context_stack: VecDeque::new(),
        }
    }

    pub fn push_context(&mut self, context_id: &str) {
        self.context_stack.push_back(context_id.to_string());
    }

    pub fn pop_context(&mut self) -> Option<String> {
        self.context_stack.pop_back()
    }
}

impl MemoryLayer for WorkingMemory {
    fn id(&self) -> &str {
        "working"
    }
    fn name(&self) -> &str {
        "Working Memory"
    }

    fn store(&mut self, key: &str, record: MemoryRecord) -> Result<(), String> {
        if self.data.len() >= self.max_capacity && !self.data.contains_key(key) {
            let oldest_key = self
                .data
                .iter()
                .min_by_key(|(_, r)| r.timestamp.clone())
                .map(|(k, _)| k.clone());
            if let Some(k) = oldest_key {
                self.data.remove(&k);
            }
        }
        self.data.insert(key.to_string(), record);
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self.data.get(key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.data.remove(key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
        let before = self.data.len();
        if self.data.len() > self.max_capacity {
            let to_remove = self.data.len() - self.max_capacity;
            let mut keys: Vec<String> = self.data.keys().cloned().collect();
            keys.sort_by_key(|k| {
                self.data
                    .get(k)
                    .map(|r| r.timestamp.clone())
                    .unwrap_or_default()
            });
            for k in keys.iter().take(to_remove) {
                self.data.remove(k);
            }
        }
        Ok(before.saturating_sub(self.data.len()))
    }

    fn search(&self, _query: &str, limit: usize) -> Vec<MemoryRecord> {
        let mut results: Vec<MemoryRecord> = self.data.values().cloned().collect();
        results.sort_by_key(|b| std::cmp::Reverse(b.priority));
        results.truncate(limit);
        results
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(100)
    }
}
