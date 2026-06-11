//! Memory layer implementations: WorkingMemory, EpisodicMemory, and FlashMemory.
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use serde_json::Value;

use super::storage::{MemoryLayer, MemoryRecord};

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

#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    episodes: Vec<MemoryRecord>,
    max_episodes: usize,
}

impl EpisodicMemory {
    pub fn new(max_episodes: usize) -> Self {
        EpisodicMemory {
            episodes: Vec::new(),
            max_episodes,
        }
    }

    pub fn recent_episodes(&self, n: usize) -> Vec<&MemoryRecord> {
        let start = self.episodes.len().saturating_sub(n);
        self.episodes[start..].iter().collect()
    }
}

impl MemoryLayer for EpisodicMemory {
    fn id(&self) -> &str {
        "episodic"
    }
    fn name(&self) -> &str {
        "Episodic Memory"
    }

    fn store(&mut self, _key: &str, mut record: MemoryRecord) -> Result<(), String> {
        record.metadata.insert(
            "episode_type".to_string(),
            Value::String("event_sequence".to_string()),
        );
        self.episodes.push(record);
        if self.episodes.len() > self.max_episodes {
            self.episodes.remove(0);
        }
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self.episodes.iter().find(|e| e.key == key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.episodes.retain(|e| e.key != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
        let before = self.episodes.len();
        if self.episodes.len() > self.max_episodes {
            let to_remove = self.episodes.len() - self.max_episodes;
            for _ in 0..to_remove {
                self.episodes.remove(0);
            }
        }
        Ok(before.saturating_sub(self.episodes.len()))
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .episodes
            .iter()
            .filter(|e| {
                e.key.to_lowercase().contains(&q)
                    || e.content
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
        self.episodes.len()
    }
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self::new(500)
    }
}

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

    fn store(&mut self, _key: &str, mut record: MemoryRecord) -> Result<(), String> {
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

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self.items.iter().find(|i| i.key == key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.items.retain(|i| i.key != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_working_memory_store_recall() {
        let mut wm = WorkingMemory::new(10);
        let record = MemoryRecord::new("k1", Value::String("v1".into()));
        wm.store("k1", record).unwrap();
        let recalled = wm.recall("k1").unwrap().unwrap();
        assert_eq!(recalled.content, "v1");
    }

    #[test]
    fn test_working_memory_recall_missing() {
        let wm = WorkingMemory::new(10);
        let result = wm.recall("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_working_memory_capacity_eviction() {
        let mut wm = WorkingMemory::new(2);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        wm.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(wm.size(), 2);
    }

    #[test]
    fn test_working_memory_update_existing_within_capacity() {
        let mut wm = WorkingMemory::new(2);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        wm.store("a", MemoryRecord::new("a", Value::Number(99.into())))
            .unwrap();
        assert_eq!(wm.size(), 2);
        let recalled = wm.recall("a").unwrap().unwrap();
        assert_eq!(recalled.content, 99);
    }

    #[test]
    fn test_working_memory_forget() {
        let mut wm = WorkingMemory::new(10);
        wm.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        wm.forget("x").unwrap();
        assert!(wm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_working_memory_compress_no_op_when_under_capacity() {
        let mut wm = WorkingMemory::new(10);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        let removed = wm.compress().unwrap();
        assert_eq!(removed, 0);
        assert_eq!(wm.size(), 2);
    }

    #[test]
    fn test_working_memory_search_by_priority() {
        let mut wm = WorkingMemory::new(10);
        wm.store(
            "low",
            MemoryRecord::new("low", Value::String("low".into())).with_priority(1),
        )
        .unwrap();
        wm.store(
            "high",
            MemoryRecord::new("high", Value::String("high".into())).with_priority(10),
        )
        .unwrap();
        let results = wm.search("irrelevant", 2);
        assert_eq!(results.len(), 2);
        assert!(results[0].priority >= results[1].priority);
    }

    #[test]
    fn test_working_memory_context_stack() {
        let mut wm = WorkingMemory::new(10);
        wm.push_context("ctx1");
        wm.push_context("ctx2");
        assert_eq!(wm.pop_context(), Some("ctx2".into()));
        assert_eq!(wm.pop_context(), Some("ctx1".into()));
        assert_eq!(wm.pop_context(), None);
    }

    #[test]
    fn test_episodic_memory_store_recall() {
        let mut em = EpisodicMemory::new(10);
        let record = MemoryRecord::new("evt1", Value::String("started".into()));
        em.store("evt1", record).unwrap();
        let recalled = em.recall("evt1").unwrap().unwrap();
        assert_eq!(recalled.content, "started");
        assert_eq!(
            recalled.metadata.get("episode_type").unwrap(),
            "event_sequence"
        );
    }

    #[test]
    fn test_episodic_memory_recall_missing() {
        let em = EpisodicMemory::new(10);
        assert!(em.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_capacity() {
        let mut em = EpisodicMemory::new(2);
        em.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        em.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        em.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(em.size(), 2);
        assert!(em.recall("a").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_recent_episodes() {
        let mut em = EpisodicMemory::new(10);
        em.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        em.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        let recent = em.recent_episodes(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].key, "b");
    }

    #[test]
    fn test_episodic_memory_forget() {
        let mut em = EpisodicMemory::new(10);
        em.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        em.forget("x").unwrap();
        assert!(em.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_search() {
        let mut em = EpisodicMemory::new(10);
        em.store(
            "hello_world",
            MemoryRecord::new("hello_world", Value::String("test data".into())),
        )
        .unwrap();
        let results = em.search("hello", 10);
        assert_eq!(results.len(), 1);
        let results = em.search("nonexistent", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_flash_memory_default() {
        let fm = FlashMemory::default();
        assert_eq!(fm.size(), 0);
    }

    #[test]
    fn test_flash_memory_capacity() {
        let mut fm = FlashMemory::new(2, 3600);
        fm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        fm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        fm.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(fm.size(), 2);
    }

    #[test]
    fn test_flash_memory_ttl_applied() {
        let mut fm = FlashMemory::new(10, 60);
        let record = MemoryRecord::new("temp", Value::String("data".into()));
        fm.store("temp", record).unwrap();
        let recalled = fm.recall("temp").unwrap().unwrap();
        assert_eq!(recalled.ttl_secs, Some(60));
    }

    #[test]
    fn test_flash_memory_priority_floor() {
        let mut fm = FlashMemory::new(10, 300);
        let record = MemoryRecord::new("low", Value::String("data".into())).with_priority(1);
        fm.store("low", record).unwrap();
        let recalled = fm.recall("low").unwrap().unwrap();
        assert!(recalled.priority >= 8);
    }

    #[test]
    fn test_flash_memory_high_priority_preserved() {
        let mut fm = FlashMemory::new(10, 300);
        let record = MemoryRecord::new("high", Value::String("data".into())).with_priority(10);
        fm.store("high", record).unwrap();
        let recalled = fm.recall("high").unwrap().unwrap();
        assert_eq!(recalled.priority, 10);
    }

    #[test]
    fn test_flash_memory_recall_missing() {
        let fm = FlashMemory::new(10, 300);
        assert!(fm.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_flash_memory_forget() {
        let mut fm = FlashMemory::new(10, 300);
        fm.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        fm.forget("x").unwrap();
        assert!(fm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_flash_memory_compress_evicts_expired() {
        let mut fm = FlashMemory::new(10, 0);
        let mut record = MemoryRecord::new("expired", Value::String("gone".into()));
        record.ttl_secs = Some(0);
        fm.store("expired", record).unwrap();
        let removed = fm.compress().unwrap();
        assert_eq!(fm.size(), 0);
    }

    #[test]
    fn test_flash_memory_search() {
        let mut fm = FlashMemory::new(10, 300);
        fm.store(
            "find_me",
            MemoryRecord::new("find_me", Value::String("data".into())),
        )
        .unwrap();
        let results = fm.search("find", 10);
        assert_eq!(results.len(), 1);
    }
}
