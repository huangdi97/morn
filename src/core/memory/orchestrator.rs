//! Memory orchestration: Experience-based memory, MemoryHub, and MemoryOrchestrator.

use crate::core::error::{MornError, MornResult};
use std::collections::HashMap;

use serde_json::Value;

use super::layers::{EpisodicMemory, FlashMemory, WorkingMemory};
use super::layers_knowledge::{GraphMemory, SemanticMemory};
use super::long_term_experience::LongTermExperience;
use super::storage::{MemoryLayer, MemoryRecord};

#[derive(Debug, Clone)]
pub struct ExperientialMemory {
    experiences: Vec<ExperienceRecord>,
    compression_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct ExperienceRecord {
    pub id: String,
    pub pattern: String,
    pub outcome: Value,
    pub frequency: u32,
    pub embedding: Option<Vec<f64>>,
}

impl ExperientialMemory {
    pub fn new(compression_threshold: usize) -> Self {
        ExperientialMemory {
            experiences: Vec::new(),
            compression_threshold,
        }
    }

    pub fn add_experience(&mut self, pattern: &str, outcome: Value, embedding: Option<Vec<f64>>) {
        if let Some(existing) = self.experiences.iter_mut().find(|e| e.pattern == pattern) {
            existing.frequency += 1;
            existing.outcome = outcome;
        } else {
            self.experiences.push(ExperienceRecord {
                id: format!("exp_{}", self.experiences.len()),
                pattern: pattern.to_string(),
                outcome,
                frequency: 1,
                embedding,
            });
        }
    }

    pub fn top_experiences(&self, n: usize) -> Vec<&ExperienceRecord> {
        let mut sorted: Vec<&ExperienceRecord> = self.experiences.iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.frequency));
        sorted.truncate(n);
        sorted
    }
}

impl MemoryLayer for ExperientialMemory {
    fn id(&self) -> &str {
        "experiential"
    }
    fn name(&self) -> &str {
        "Experiential Memory"
    }

    fn store(&mut self, _key: &str, record: MemoryRecord) -> Result<(), MornError> {
        let pattern = record.content.as_str().unwrap_or(&record.key).to_string();
        let outcome = record
            .metadata
            .get("outcome")
            .cloned()
            .unwrap_or(record.content);
        self.add_experience(&pattern, outcome, None);
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, MornError> {
        Ok(self
            .experiences
            .iter()
            .find(|e| e.id == key || e.pattern == key)
            .map(|e| {
                MemoryRecord::new(&e.id, e.outcome.clone())
                    .with_metadata("frequency", Value::Number(e.frequency.into()))
                    .with_metadata("pattern", Value::String(e.pattern.clone()))
            }))
    }

    fn forget(&mut self, key: &str) -> Result<(), MornError> {
        self.experiences.retain(|e| e.id != key && e.pattern != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, MornError> {
        let before = self.experiences.len();
        if self.experiences.len() > self.compression_threshold {
            self.experiences.sort_by_key(|a| a.frequency);
            let to_remove = self.experiences.len() - self.compression_threshold;
            self.experiences.drain(..to_remove);
        }
        Ok(before.saturating_sub(self.experiences.len()))
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .experiences
            .iter()
            .filter(|e| e.pattern.to_lowercase().contains(&q))
            .map(|e| {
                MemoryRecord::new(&e.id, e.outcome.clone())
                    .with_metadata("frequency", Value::Number(e.frequency.into()))
            })
            .collect();
        results.truncate(limit);
        results
    }

    fn size(&self) -> usize {
        self.experiences.len()
    }
}

impl Default for ExperientialMemory {
    fn default() -> Self {
        Self::new(1000)
    }
}

pub struct MemoryHub {
    layers: HashMap<String, Box<dyn MemoryLayer>>,
}

impl MemoryHub {
    pub fn new() -> Self {
        let mut hub = MemoryHub {
            layers: HashMap::new(),
        };
        hub.register(Box::new(WorkingMemory::default()));
        hub.register(Box::new(EpisodicMemory::default()));
        hub.register(Box::new(SemanticMemory::default()));
        hub.register(Box::new(ExperientialMemory::default()));
        hub.register(Box::new(GraphMemory::default()));
        hub.register(Box::new(FlashMemory::default()));
        hub.register(Box::new(LongTermExperience::default()));
        hub
    }

    pub fn register(&mut self, layer: Box<dyn MemoryLayer>) {
        let id = layer.id().to_string();
        self.layers.insert(id, layer);
    }

    pub fn get(&self, id: &str) -> Option<&dyn MemoryLayer> {
        self.layers.get(id).map(|b| b.as_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn MemoryLayer> {
        let layer = self.layers.get_mut(id)?;
        Some(layer.as_mut())
    }

    pub fn store_all(&mut self, key: &str, record: MemoryRecord) -> Vec<Result<(), MornError>> {
        self.layers
            .iter_mut()
            .map(|(_, layer)| layer.store(key, record.clone()))
            .collect()
    }

    pub fn search_all(&self, query: &str, limit: usize) -> HashMap<String, Vec<MemoryRecord>> {
        self.layers
            .iter()
            .map(|(id, layer)| (id.clone(), layer.search(query, limit)))
            .collect()
    }

    pub fn compress_all(&mut self) -> HashMap<String, usize> {
        self.layers
            .iter_mut()
            .map(|(id, layer)| (id.clone(), layer.compress().unwrap_or(0)))
            .collect()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for MemoryHub {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemoryOrchestrator {
    hub: MemoryHub,
}

impl MemoryOrchestrator {
    pub fn new(hub: MemoryHub) -> Self {
        MemoryOrchestrator { hub }
    }

    pub fn decide_with_memory(
        &mut self,
        context: &str,
    ) -> MornResult<HashMap<String, Vec<MemoryRecord>>> {
        let working = self.hub.get("working").map(|m| m.search(context, 10));
        let episodic = self.hub.get("episodic").map(|m| m.search(context, 5));
        let semantic = self.hub.get("semantic").map(|m| m.search(context, 5));
        let experiential = self.hub.get("experiential").map(|m| m.search(context, 5));
        let graph = self.hub.get("graph").map(|m| m.search(context, 5));
        let flash = self.hub.get("flash").map(|m| m.search(context, 5));
        let long_term = self
            .hub
            .get("long_term_experience")
            .map(|m| m.search(context, 5));

        let mut results = HashMap::new();
        if let Some(r) = working {
            results.insert("working".to_string(), r);
        }
        if let Some(r) = episodic {
            results.insert("episodic".to_string(), r);
        }
        if let Some(r) = semantic {
            results.insert("semantic".to_string(), r);
        }
        if let Some(r) = experiential {
            results.insert("experiential".to_string(), r);
        }
        if let Some(r) = graph {
            results.insert("graph".to_string(), r);
        }
        if let Some(r) = flash {
            results.insert("flash".to_string(), r);
        }
        if let Some(r) = long_term {
            results.insert("long_term_experience".to_string(), r);
        }
        Ok(results)
    }

    pub fn hub(&self) -> &MemoryHub {
        &self.hub
    }

    pub fn hub_mut(&mut self) -> &mut MemoryHub {
        &mut self.hub
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_memory_hub_default_layers() {
        let hub = MemoryHub::new();
        assert_eq!(hub.layer_count(), 7);
    }

    #[test]
    fn test_memory_hub_register() {
        let hub = MemoryHub::new();
        assert!(hub.get("working").is_some());
        assert!(hub.get("episodic").is_some());
        assert!(hub.get("semantic").is_some());
        assert!(hub.get("experiential").is_some());
        assert!(hub.get("graph").is_some());
        assert!(hub.get("flash").is_some());
        assert!(hub.get("long_term_experience").is_some());
    }

    #[test]
    fn test_memory_hub_get_missing() {
        let hub = MemoryHub::new();
        assert!(hub.get("nonexistent").is_none());
    }

    #[test]
    fn test_memory_hub_get_mut() {
        let mut hub = MemoryHub::new();
        let wm = hub.get_mut("working").unwrap();
        assert!(wm.id() == "working");
    }

    #[test]
    fn test_memory_hub_store_all() {
        let mut hub = MemoryHub::new();
        let results = hub.store_all(
            "test_key",
            MemoryRecord::new("test_key", Value::String("val".into())),
        );
        assert_eq!(results.len(), 7);
        for r in &results {
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_memory_hub_search_all() {
        let mut hub = MemoryHub::new();
        hub.store_all(
            "query_term",
            MemoryRecord::new("query_term", Value::String("data".into())),
        );
        let results = hub.search_all("query_term", 10);
        assert_eq!(results.len(), 7);
        assert!(results.contains_key("working"));
    }

    #[test]
    fn test_memory_hub_compress_all() {
        let mut hub = MemoryHub::new();
        for i in 0..150 {
            hub.store_all(
                &format!("k{}", i),
                MemoryRecord::new(&format!("k{}", i), Value::Number(i.into())),
            );
        }
        let compressed = hub.compress_all();
        assert!(compressed.contains_key("working"));
        assert!(compressed.contains_key("episodic"));
    }

    #[test]
    fn test_experiential_memory_store_recall() {
        let mut xm = ExperientialMemory::default();
        let record = MemoryRecord::new("pattern1", Value::String("outcome1".into()))
            .with_metadata("outcome", Value::String("success".into()));
        xm.store("", record).unwrap();
        let recalled = xm.recall("outcome1").unwrap().unwrap();
        assert_eq!(recalled.metadata.get("pattern").unwrap(), "outcome1");
    }

    #[test]
    fn test_experiential_memory_frequency_increment() {
        let mut xm = ExperientialMemory::default();
        xm.add_experience("retry", Value::String("success".into()), None);
        xm.add_experience("retry", Value::String("success".into()), None);
        let top = xm.top_experiences(1);
        assert_eq!(top[0].frequency, 2);
    }

    #[test]
    fn test_experiential_memory_compress() {
        let mut xm = ExperientialMemory::new(2);
        xm.add_experience("a", Value::String("1".into()), None);
        xm.add_experience("b", Value::String("2".into()), None);
        xm.add_experience("c", Value::String("3".into()), None);
        let removed = xm.compress().unwrap();
        assert!(removed > 0);
        assert_eq!(xm.size(), 2);
    }

    #[test]
    fn test_experiential_memory_forget() {
        let mut xm = ExperientialMemory::default();
        xm.add_experience("pattern1", Value::String("outcome".into()), None);
        xm.forget("pattern1").unwrap();
        assert!(xm.recall("pattern1").unwrap().is_none());
    }

    #[test]
    fn test_experiential_memory_search() {
        let mut xm = ExperientialMemory::default();
        xm.add_experience("login_failed", Value::String("retry".into()), None);
        let results = xm.search("login", 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_orchestrator_new() {
        let hub = MemoryHub::new();
        let orchestrator = MemoryOrchestrator::new(hub);
        assert_eq!(orchestrator.hub().layer_count(), 7);
    }

    #[test]
    fn test_memory_orchestrator_decide() {
        let hub = MemoryHub::new();
        let mut orchestrator = MemoryOrchestrator::new(hub);
        orchestrator.hub_mut().store_all(
            "context_key",
            MemoryRecord::new("context_key", Value::String("data".into())),
        );
        let results = orchestrator.decide_with_memory("context_key").unwrap();
        assert_eq!(results.len(), 7);
    }

    #[test]
    fn test_memory_orchestrator_decide_no_match() {
        let hub = MemoryHub::new();
        let mut orchestrator = MemoryOrchestrator::new(hub);
        let results = orchestrator
            .decide_with_memory("zzz_nonexistent_zzz")
            .unwrap();
        for v in results.values() {
            assert!(v.is_empty());
        }
    }

    #[test]
    fn test_experiential_memory_default() {
        let xm = ExperientialMemory::default();
        assert_eq!(xm.size(), 0);
    }
}
