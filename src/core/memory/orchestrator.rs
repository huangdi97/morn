//! Memory orchestration: Experience-based memory, MemoryHub, and MemoryOrchestrator.

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

    fn store(&mut self, _key: &str, record: MemoryRecord) -> Result<(), String> {
        let pattern = record.content.as_str().unwrap_or(&record.key).to_string();
        let outcome = record
            .metadata
            .get("outcome")
            .cloned()
            .unwrap_or(record.content);
        self.add_experience(&pattern, outcome, None);
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
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

    fn forget(&mut self, key: &str) -> Result<(), String> {
        self.experiences.retain(|e| e.id != key && e.pattern != key);
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
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

    pub fn store_all(&mut self, key: &str, record: MemoryRecord) -> Vec<Result<(), String>> {
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
    ) -> Result<HashMap<String, Vec<MemoryRecord>>, String> {
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
