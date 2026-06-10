//! Structured knowledge with record-based field matching.

use std::collections::HashMap;

use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

use super::{Knowledge, KnowledgeItem};

#[derive(Debug, Clone)]
pub struct StructuredKnowledge {
    id: String,
    _name: String,
    records: Vec<HashMap<String, String>>,
}

impl StructuredKnowledge {
    pub fn new(id: &str, name: &str) -> Self {
        StructuredKnowledge {
            id: id.to_string(),
            _name: name.to_string(),
            records: Vec::new(),
        }
    }
}

impl Component for StructuredKnowledge {
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

impl IOComponent for StructuredKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "field query".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "json".into(),
                description: "matched records".into(),
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

impl SecureComponent for StructuredKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Knowledge for StructuredKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let lower = query.to_lowercase();
        let mut results = Vec::new();
        for (i, record) in self.records.iter().enumerate() {
            for (field, value) in record {
                if field.to_lowercase().contains(&lower)
                    || value.to_lowercase().contains(&lower)
                {
                    results.push(KnowledgeItem {
                        key: format!("record[{}].{}", i, field),
                        value: value.clone(),
                        source: "structured".to_string(),
                    });
                }
            }
        }
        Ok(results)
    }

    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String> {
        let mut record = HashMap::new();
        for item in items {
            record.insert(item.key, item.value);
        }
        self.records.push(record);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_knowledge_query() {
        let mut sk = StructuredKnowledge::new("struct-1", "TestStruct");
        sk.update(vec![
            KnowledgeItem {
                key: "name".into(),
                value: "Alice".into(),
                source: "test".into(),
            },
            KnowledgeItem {
                key: "role".into(),
                value: "Engineer".into(),
                source: "test".into(),
            },
        ]);
        let results = sk.query("Alice").unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_structured_knowledge_field_query() {
        let mut sk = StructuredKnowledge::new("struct-2", "TestStruct2");
        sk.update(vec![
            KnowledgeItem {
                key: "name".into(),
                value: "Bob".into(),
                source: "test".into(),
            },
            KnowledgeItem {
                key: "age".into(),
                value: "30".into(),
                source: "test".into(),
            },
            KnowledgeItem {
                key: "city".into(),
                value: "Beijing".into(),
                source: "test".into(),
            },
        ]);
        let results = sk.query("Bob").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "Bob");
    }

    #[test]
    fn test_structured_knowledge_no_match() {
        let sk = StructuredKnowledge::new("struct-3", "EmptyStruct");
        let results = sk.query("nonexistent").unwrap();
        assert!(results.is_empty());
    }
}
