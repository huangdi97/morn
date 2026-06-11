//! 长期经验记忆 — 跨会话知识沉淀与模式学习
use serde_json::Value;
use std::collections::HashMap;

use super::storage::{MemoryLayer, MemoryRecord};

#[derive(Debug, Clone)]
pub struct Experience {
    pub id: String,
    pub summary: String,
    pub importance: f64,
    pub created_at: i64,
    pub access_count: u64,
}

#[derive(Debug, Clone)]
pub struct LongTermExperience {
    experiences: Vec<Experience>,
    max_experiences: usize,
    index: HashMap<String, usize>,
}

impl LongTermExperience {
    pub fn new(max_experiences: usize) -> Self {
        LongTermExperience {
            experiences: Vec::new(),
            max_experiences,
            index: HashMap::new(),
        }
    }

    pub fn add_experience(&mut self, summary: &str, importance: f64) -> String {
        let id = format!(
            "lte_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let exp = Experience {
            id: id.clone(),
            summary: summary.to_string(),
            importance,
            created_at: chrono::Utc::now().timestamp(),
            access_count: 0,
        };
        self.index.insert(id.clone(), self.experiences.len());
        self.experiences.push(exp);
        self.enforce_capacity();
        id
    }

    pub fn access_experience(&mut self, id: &str) -> Option<&Experience> {
        if let Some(&idx) = self.index.get(id) {
            self.experiences[idx].access_count += 1;
            Some(&self.experiences[idx])
        } else {
            None
        }
    }

    pub fn top_experiences(&self, n: usize) -> Vec<&Experience> {
        let mut sorted: Vec<&Experience> = self.experiences.iter().collect();
        sorted.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(n);
        sorted
    }

    fn enforce_capacity(&mut self) {
        while self.experiences.len() > self.max_experiences {
            let idx = self
                .experiences
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.importance
                        .partial_cmp(&b.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i);
            if let Some(remove_idx) = idx {
                let removed = self.experiences.swap_remove(remove_idx);
                self.index.remove(&removed.id);
                if remove_idx < self.experiences.len() {
                    self.index
                        .insert(self.experiences[remove_idx].id.clone(), remove_idx);
                }
            }
        }
    }
}

impl MemoryLayer for LongTermExperience {
    fn id(&self) -> &str {
        "long_term_experience"
    }

    fn name(&self) -> &str {
        "Long-Term Experience"
    }

    fn store(&mut self, _key: &str, record: MemoryRecord) -> Result<(), String> {
        let summary = record.content.as_str().unwrap_or(&record.key).to_string();
        let importance = record
            .metadata
            .get("importance")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        self.add_experience(&summary, importance);
        Ok(())
    }

    fn recall(&self, key: &str) -> Result<Option<MemoryRecord>, String> {
        Ok(self
            .experiences
            .iter()
            .find(|e| e.id == key || e.summary == key)
            .map(|e| {
                MemoryRecord::new(&e.id, Value::String(e.summary.clone()))
                    .with_metadata(
                        "importance",
                        Value::Number(
                            serde_json::Number::from_f64(e.importance)
                                .unwrap_or(serde_json::Number::from(0)),
                        ),
                    )
                    .with_metadata("created_at", Value::Number(e.created_at.into()))
                    .with_metadata("access_count", Value::Number(e.access_count.into()))
            }))
    }

    fn forget(&mut self, key: &str) -> Result<(), String> {
        if let Some(&idx) = self.index.get(key) {
            let removed = self.experiences.swap_remove(idx);
            self.index.remove(&removed.id);
            if idx < self.experiences.len() {
                self.index.insert(self.experiences[idx].id.clone(), idx);
            }
        }
        Ok(())
    }

    fn compress(&mut self) -> Result<usize, String> {
        let before = self.experiences.len();
        self.enforce_capacity();
        Ok(before.saturating_sub(self.experiences.len()))
    }

    fn search(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let q = query.to_lowercase();
        let mut results: Vec<MemoryRecord> = self
            .experiences
            .iter()
            .filter(|e| e.summary.to_lowercase().contains(&q) || e.id.to_lowercase().contains(&q))
            .map(|e| {
                MemoryRecord::new(&e.id, Value::String(e.summary.clone()))
                    .with_metadata(
                        "importance",
                        Value::Number(
                            serde_json::Number::from_f64(e.importance)
                                .unwrap_or(serde_json::Number::from(0)),
                        ),
                    )
                    .with_metadata("access_count", Value::Number(e.access_count.into()))
            })
            .collect();
        results.truncate(limit);
        results
    }

    fn size(&self) -> usize {
        self.experiences.len()
    }
}

impl Default for LongTermExperience {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_term_experience_store_recall() {
        let mut lte = LongTermExperience::default();
        let record = MemoryRecord::new("test_exp", Value::String("important lesson".into()))
            .with_metadata(
                "importance",
                Value::Number(serde_json::Number::from_f64(0.9).unwrap()),
            );
        lte.store("test_exp", record).unwrap();
        assert_eq!(lte.size(), 1);
    }

    #[test]
    fn test_long_term_experience_recall_by_summary() {
        let mut lte = LongTermExperience::default();
        let record = MemoryRecord::new("exp1", Value::String("my summary".into())).with_metadata(
            "importance",
            Value::Number(serde_json::Number::from_f64(0.8).unwrap()),
        );
        lte.store("exp1", record).unwrap();

        let recalled = lte.recall("my summary").unwrap().unwrap();
        assert_eq!(recalled.content, "my summary");
        assert!(recalled.metadata.contains_key("importance"));
        assert!(recalled.metadata.contains_key("created_at"));
        assert!(recalled.metadata.contains_key("access_count"));
    }

    #[test]
    fn test_long_term_experience_recall_missing() {
        let lte = LongTermExperience::default();
        assert!(lte.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_long_term_experience_compress_no_op() {
        let mut lte = LongTermExperience::new(10);
        let r1 = MemoryRecord::new("a", Value::String("low".into())).with_metadata(
            "importance",
            Value::Number(serde_json::Number::from_f64(0.1).unwrap()),
        );
        let r2 = MemoryRecord::new("b", Value::String("medium".into())).with_metadata(
            "importance",
            Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
        );
        lte.store("a", r1).unwrap();
        lte.store("b", r2).unwrap();
        let removed = lte.compress().unwrap();
        assert_eq!(removed, 0);
        assert_eq!(lte.size(), 2);
    }

    #[test]
    fn test_long_term_experience_add_experience() {
        let mut lte = LongTermExperience::new(10);
        let id = lte.add_experience("test summary", 0.7);
        assert!(id.starts_with("lte_"));
        assert_eq!(lte.size(), 1);
    }

    #[test]
    fn test_long_term_experience_access_experience() {
        let mut lte = LongTermExperience::new(10);
        let id = lte.add_experience("my exp", 0.6);
        let exp = lte.access_experience(&id).unwrap();
        assert_eq!(exp.access_count, 1);
        let exp = lte.access_experience(&id).unwrap();
        assert_eq!(exp.access_count, 2);
    }

    #[test]
    fn test_long_term_experience_access_missing() {
        let mut lte = LongTermExperience::new(10);
        assert!(lte.access_experience("nonexistent").is_none());
    }

    #[test]
    fn test_long_term_experience_top_experiences() {
        let mut lte = LongTermExperience::new(10);
        lte.add_experience("low", 0.1);
        lte.add_experience("high", 0.9);
        lte.add_experience("medium", 0.5);
        let top = lte.top_experiences(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].summary, "high");
        assert_eq!(top[1].summary, "medium");
    }

    #[test]
    fn test_long_term_experience_forget() {
        let mut lte = LongTermExperience::new(10);
        let id = lte.add_experience("to forget", 0.5);
        lte.forget(&id).unwrap();
        assert!(lte.recall(&id).unwrap().is_none());
        assert_eq!(lte.size(), 0);
    }

    #[test]
    fn test_long_term_experience_search() {
        let mut lte = LongTermExperience::new(10);
        lte.add_experience("important finding", 0.8);
        let results = lte.search("finding", 10);
        assert_eq!(results.len(), 1);
        let results = lte.search("nothing", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_long_term_experience_enforce_capacity() {
        let mut lte = LongTermExperience::new(1);
        lte.add_experience("first", 0.5);
        lte.add_experience("second", 0.9);
        assert_eq!(lte.size(), 1);
    }
}
