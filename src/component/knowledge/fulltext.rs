//! Fulltext knowledge with inverted-index keyword search.

use std::collections::{BTreeSet, HashMap};

use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

use super::{Knowledge, KnowledgeItem};

#[derive(Debug, Clone)]
pub struct FulltextKnowledge {
    id: String,
    _name: String,
    index: HashMap<String, Vec<String>>,
    docs: HashMap<String, String>,
}

impl FulltextKnowledge {
    pub fn new(id: &str, name: &str) -> Self {
        FulltextKnowledge {
            id: id.to_string(),
            _name: name.to_string(),
            index: HashMap::new(),
            docs: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, key: &str, text: &str) {
        self.docs.insert(key.to_string(), text.to_string());
        for word in text.split_whitespace() {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if !cleaned.is_empty() {
                self.index
                    .entry(cleaned.to_lowercase())
                    .or_default()
                    .push(key.to_string());
            }
        }
    }
}

impl Component for FulltextKnowledge {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "knowledge"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for FulltextKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "keyword search".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "json".into(),
                description: "matching docs".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for FulltextKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Knowledge for FulltextKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let lower = query.to_lowercase();
        let mut matched = BTreeSet::new();
        for word in lower.split_whitespace() {
            if let Some(doc_keys) = self.index.get(word) {
                for k in doc_keys {
                    matched.insert(k.clone());
                }
            }
        }
        Ok(matched
            .into_iter()
            .map(|key| KnowledgeItem {
                key: key.clone(),
                value: self.docs.get(&key).cloned().unwrap_or_default(),
                source: "fulltext".to_string(),
            })
            .collect())
    }

    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String> {
        for item in items {
            self.add_document(&item.key, &item.value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fulltext_knowledge_indexing() {
        let mut ftk = FulltextKnowledge::new("ft-1", "TestFT");
        ftk.add_document("doc1", "the quick brown fox");
        ftk.add_document("doc2", "jumps over the lazy dog");
        let results = ftk.query("fox").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "doc1");
    }

    #[test]
    fn test_fulltext_knowledge_multi_word_query() {
        let mut ftk = FulltextKnowledge::new("ft-2", "TestFT2");
        ftk.add_document("doc1", "Rust programming language");
        ftk.add_document("doc2", "Python programming language");
        let results = ftk.query("Rust programming").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_fulltext_knowledge_keyword_search() {
        let mut ftk = FulltextKnowledge::new("ft-3", "TestFT3");
        ftk.add_document("readme", "Rust is a systems programming language");
        ftk.add_document("guide", "Python is a high-level programming language");
        let results = ftk.query("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].value.contains("Rust"));
    }

    #[test]
    fn test_fulltext_knowledge_no_match() {
        let ftk = FulltextKnowledge::new("ft-4", "EmptyFT");
        let results = ftk.query("missing").unwrap();
        assert!(results.is_empty());
    }
}
