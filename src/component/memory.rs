use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use std::collections::HashMap;

pub trait Memory: IOComponent {
    fn store(&mut self, key: &str, value: &str, namespace: &str) -> Result<(), String>;
    fn retrieve(&self, key: &str, namespace: &str) -> Result<Option<String>, String>;
    fn search(&self, query: &str, namespace: &str) -> Result<Vec<(String, String)>, String>;
}

#[allow(dead_code)]
pub struct SqliteMemory {
    id: String,
    name: String,
    data: HashMap<String, HashMap<String, String>>,
}

impl SqliteMemory {
    pub fn new() -> Self {
        SqliteMemory {
            id: "memory-sqlite".into(),
            name: "SQLite Memory".into(),
            data: HashMap::new(),
        }
    }
}

impl Component for SqliteMemory {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "memory"
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

impl IOComponent for SqliteMemory {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "store".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "key:value:namespace".into(),
            },
            Port {
                id: "retrieve".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "stored value".into(),
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

impl SecureComponent for SqliteMemory {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Memory for SqliteMemory {
    fn store(&mut self, key: &str, value: &str, namespace: &str) -> Result<(), String> {
        self.data
            .entry(namespace.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn retrieve(&self, key: &str, namespace: &str) -> Result<Option<String>, String> {
        Ok(self.data.get(namespace).and_then(|ns| ns.get(key)).cloned())
    }

    fn search(&self, query: &str, namespace: &str) -> Result<Vec<(String, String)>, String> {
        let mut results = Vec::new();
        if let Some(ns) = self.data.get(namespace) {
            for (k, v) in ns {
                if k.contains(query) || v.contains(query) {
                    results.push((k.clone(), v.clone()));
                }
            }
        }
        Ok(results)
    }
}

pub fn create_default_memory() -> Box<dyn Memory> {
    Box::new(SqliteMemory::new())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub namespace: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoreMemory {
    items: Vec<MemoryItem>,
}

impl CoreMemory {
    pub fn new() -> Self {
        CoreMemory { items: Vec::new() }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.items.push(MemoryItem {
            key: key.to_string(),
            value: value.to_string(),
            namespace: "core".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn get(&self, key: &str) -> Option<&MemoryItem> {
        self.items.iter().find(|i| i.key == key)
    }

    pub fn remove(&mut self, key: &str) {
        self.items.retain(|i| i.key != key);
    }

    pub fn to_prompt_block(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }
        let mut block = "## Core Memory\n".to_string();
        for item in &self.items {
            block.push_str(&format!("- {}: {}\n", item.key, item.value));
        }
        block
    }

    pub fn apply_changes(&mut self, changes: Vec<MemoryChange>) -> Result<(), String> {
        for change in changes {
            match change.operation.as_str() {
                "add" => self.add(&change.key, &change.value),
                "set" => {
                    self.remove(&change.key);
                    self.add(&change.key, &change.value);
                }
                "remove" | "delete" => self.remove(&change.key),
                _ => return Err(format!("Unknown memory operation: {}", change.operation)),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecallMemory {
    items: Vec<MemoryItem>,
}

impl RecallMemory {
    pub fn new() -> Self {
        RecallMemory { items: Vec::new() }
    }

    pub fn store(&mut self, key: &str, value: &str, namespace: &str) {
        self.items.push(MemoryItem {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryItem> {
        let lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|i| {
                i.key.to_lowercase().contains(&lower)
                    || i.value.to_lowercase().contains(&lower)
                    || i.namespace.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn all(&self) -> &[MemoryItem] {
        &self.items
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchivalMemory {
    items: Vec<MemoryItem>,
}

impl ArchivalMemory {
    pub fn new() -> Self {
        ArchivalMemory { items: Vec::new() }
    }

    pub fn store(&mut self, content: &str) {
        self.items.push(MemoryItem {
            key: format!("archival-{}", self.items.len()),
            value: content.to_string(),
            namespace: "archival".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryItem> {
        let lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|i| i.value.to_lowercase().contains(&lower))
            .collect()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryChange {
    pub operation: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

pub struct MemoryManager {
    core: CoreMemory,
    recall: RecallMemory,
    archival: ArchivalMemory,
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            core: CoreMemory::new(),
            recall: RecallMemory::new(),
            archival: ArchivalMemory::new(),
        }
    }

    pub fn to_prompt_block(&self) -> String {
        let mut block = self.core.to_prompt_block();
        if !block.is_empty() {
            block.push('\n');
        }
        if !self.recall.all().is_empty() {
            block.push_str("## Recent Memories\n");
            for item in self.recall.all().iter().rev().take(5) {
                block.push_str(&format!(
                    "- [{}] {}: {}\n",
                    item.namespace, item.key, item.value
                ));
            }
            block.push('\n');
        }
        block
    }

    pub fn search(&self, query: &str) -> Result<Vec<MemoryItem>, String> {
        let mut results = Vec::new();
        for item in self.recall.search(query) {
            results.push(item.clone());
        }
        for item in self.archival.search(query) {
            results.push(item.clone());
        }
        Ok(results)
    }

    pub fn store_archival(&self, _content: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn store_recall(&mut self, key: &str, value: &str, namespace: &str) {
        self.recall.store(key, value, namespace);
    }

    pub fn store_core(&mut self, key: &str, value: &str) {
        self.core.add(key, value);
    }

    pub async fn agent_edit_core(&mut self, changes: Vec<MemoryChange>) -> Result<(), String> {
        self.core.apply_changes(changes)
    }

    pub async fn extract_memories(
        &mut self,
        conversation: &[ConversationMessage],
    ) -> Result<Vec<MemoryItem>, String> {
        let mut extracted = Vec::new();
        for msg in conversation {
            let content_lower = msg.content.to_lowercase();
            if content_lower.contains("remember that")
                || content_lower.contains("remember:")
                || content_lower.contains("my name is")
                || content_lower.contains("i am")
                || content_lower.contains("my name")
            {
                let item = MemoryItem {
                    key: format!("extracted-{}", extracted.len()),
                    value: msg.content.clone(),
                    namespace: "extracted".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                self.recall.store(&item.key, &item.value, &item.namespace);
                extracted.push(item);
            }
        }
        Ok(extracted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_memory_store_retrieve() {
        let mut mem = SqliteMemory::new();
        mem.store("key1", "value1", "ns1").unwrap();
        let val = mem.retrieve("key1", "ns1").unwrap();
        assert_eq!(val, Some("value1".to_string()));
    }

    #[test]
    fn test_sqlite_memory_search() {
        let mut mem = SqliteMemory::new();
        mem.store("name", "Alice", "users").unwrap();
        mem.store("email", "alice@test.com", "users").unwrap();
        let results = mem.search("Alice", "users").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_core_memory() {
        let mut core = CoreMemory::new();
        core.add("name", "Alice");
        assert_eq!(core.get("name").unwrap().value, "Alice");
        assert!(core.to_prompt_block().contains("Alice"));
    }

    #[test]
    fn test_core_memory_remove() {
        let mut core = CoreMemory::new();
        core.add("key1", "val1");
        core.remove("key1");
        assert!(core.get("key1").is_none());
    }

    #[test]
    fn test_recall_memory_search() {
        let mut recall = RecallMemory::new();
        recall.store("project", "Morn AI", "work");
        let results = recall.search("Morn");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_archival_memory() {
        let mut archival = ArchivalMemory::new();
        archival.store("Important research data about AI");
        let results = archival.search("AI");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_manager_to_prompt() {
        let mut mgr = MemoryManager::new();
        mgr.store_core("role", "assistant");
        let prompt = mgr.to_prompt_block();
        assert!(prompt.contains("role"));
    }

    #[test]
    fn test_memory_manager_search() {
        let mut mgr = MemoryManager::new();
        mgr.store_recall("project", "Morn", "work");
        let results = mgr.search("Morn").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_edit_core() {
        let mut mgr = MemoryManager::new();
        mgr.store_core("name", "Bot");
        let changes = vec![MemoryChange {
            operation: "set".to_string(),
            key: "name".to_string(),
            value: "UpdatedBot".to_string(),
        }];
        mgr.agent_edit_core(changes).await.unwrap();
        assert_eq!(mgr.core.get("name").unwrap().value, "UpdatedBot");
    }

    #[tokio::test]
    async fn test_extract_memories() {
        let mut mgr = MemoryManager::new();
        let conv = vec![
            ConversationMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ConversationMessage {
                role: "user".to_string(),
                content: "My name is Alice".to_string(),
            },
            ConversationMessage {
                role: "user".to_string(),
                content: "Remember that I like Python".to_string(),
            },
        ];
        let extracted = mgr.extract_memories(&conv).await.unwrap();
        assert_eq!(extracted.len(), 2);
    }

    #[test]
    fn test_core_memory_apply_changes() {
        let mut core = CoreMemory::new();
        let changes = vec![
            MemoryChange {
                operation: "add".to_string(),
                key: "lang".to_string(),
                value: "Rust".to_string(),
            },
            MemoryChange {
                operation: "add".to_string(),
                key: "version".to_string(),
                value: "1.0".to_string(),
            },
        ];
        core.apply_changes(changes).unwrap();
        assert_eq!(core.get("lang").unwrap().value, "Rust");
        assert_eq!(core.get("version").unwrap().value, "1.0");
    }

    #[test]
    fn test_empty_core_prompt() {
        let core = CoreMemory::new();
        assert!(core.to_prompt_block().is_empty());
    }
}
