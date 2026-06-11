use serde_json::Value;

use super::super::storage::{MemoryLayer, MemoryRecord};

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
