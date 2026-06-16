//! Flash memory — priority-tagged instant recall.
use crate::core::error::MornError;
use std::collections::VecDeque;
use std::time::Duration;

use super::super::storage::{MemoryLayer, MemoryRecord};

#[derive(Debug, Clone)]
pub struct FlashMemory {
    items: VecDeque<MemoryRecord>,
    max_capacity: usize,
    default_ttl: Duration,
}

impl FlashMemory {
    pub fn new(max_capacity: usize, default_ttl_secs: u64) -> Self {
        FlashMemory {
            items: VecDeque::new(),
            max_capacity,
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }

    pub fn evict_expired(&mut self) {
        let now = chrono::Utc::now();
        self.items.retain(|item| {
            if let Some(ttl) = item.ttl_secs {
                let created = chrono::DateTime::parse_from_rfc3339(&item.timestamp)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or(now);
                now.signed_duration_since(created)
                    .num_seconds()
                    .unsigned_abs()
                    < ttl
            } else {
                true
            }
        });
    }
}

impl MemoryLayer for FlashMemory {
    fn id(&self) -> &str {
        "flash"
    }
    fn name(&self) -> &str {
        "Flash Memory"
    }

    fn store(&mut self, _key: &str, mut record: MemoryRecord) -> Result<(), MornError> {
        if record.ttl_secs.is_none() {
            record.ttl_secs = Some(self.default_ttl.as_secs());
        }
        if record.priority < 8 {
            record.priority = 8;
        }
        self.items.push_back(record);
        if self.items.len() > self.max_capacity {
            self.items.pop_front();
        }
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, MornError> {
        Ok(self.items.iter().find(|i| i.key == key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), MornError> {
        self.items.retain(|i| i.key != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, MornError> {
        let before = self.items.len();
        self.evict_expired();
        if self.items.len() > self.max_capacity {
            let to_remove = self.items.len() - self.max_capacity;
            for _ in 0..to_remove {
                self.items.pop_front();
            }
        }
        Ok(before.saturating_sub(self.items.len()))
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .items
            .iter()
            .filter(|i| {
                i.key.to_lowercase().contains(&q)
                    || i.content
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
        self.items.len()
    }
}

impl Default for FlashMemory {
    fn default() -> Self {
        Self::new(50, 300)
    }
}
