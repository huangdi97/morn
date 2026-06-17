//! storage — Defines serializable memory entries and storage records.
use crate::core::error::MornError;
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

    pub fn apply_changes(&mut self, changes: Vec<MemoryChange>) -> Result<(), MornError> {
        for change in changes {
            match change.operation.as_str() {
                "add" => self.add(&change.key, &change.value),
                "set" => {
                    self.remove(&change.key);
                    self.add(&change.key, &change.value);
                }
                "remove" | "delete" => self.remove(&change.key),
                _ => {
                    return Err(MornError::Internal(format!(
                        "Unknown memory operation: {}",
                        change.operation
                    )))
                }
            }
        }
        Ok(())
    }
}

impl Default for CoreMemory {
    fn default() -> Self {
        Self::new()
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

impl Default for RecallMemory {
    fn default() -> Self {
        Self::new()
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

impl Default for ArchivalMemory {
    fn default() -> Self {
        Self::new()
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
    pub(super) core: CoreMemory,
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

    pub fn search(&self, query: &str) -> Result<Vec<MemoryItem>, MornError> {
        let mut results = Vec::new();
        for item in self.recall.search(query) {
            results.push(item.clone());
        }
        for item in self.archival.search(query) {
            results.push(item.clone());
        }
        Ok(results)
    }

    pub fn store_archival(&self, _content: &str) -> Result<(), MornError> {
        Ok(())
    }

    pub fn store_recall(&mut self, key: &str, value: &str, namespace: &str) {
        self.recall.store(key, value, namespace);
    }

    pub fn store_core(&mut self, key: &str, value: &str) {
        self.core.add(key, value);
    }

    pub async fn agent_edit_core(&mut self, changes: Vec<MemoryChange>) -> Result<(), MornError> {
        self.core.apply_changes(changes)
    }

    pub async fn extract_memories(
        &mut self,
        conversation: &[ConversationMessage],
    ) -> Result<Vec<MemoryItem>, MornError> {
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

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_memory_new_empty() {
        let m = CoreMemory::new();
        assert_eq!(m.to_prompt_block(), "");
    }

    #[test]
    fn test_core_memory_add_and_get() {
        let mut m = CoreMemory::new();
        m.add("name", "test-user");
        let item = m.get("name").unwrap();
        assert_eq!(item.key, "name");
        assert_eq!(item.value, "test-user");
    }

    #[test]
    fn test_core_memory_get_missing() {
        let m = CoreMemory::new();
        assert!(m.get("missing").is_none());
    }

    #[test]
    fn test_core_memory_remove() {
        let mut m = CoreMemory::new();
        m.add("key1", "val1");
        m.add("key2", "val2");
        m.remove("key1");
        assert!(m.get("key1").is_none());
        assert!(m.get("key2").is_some());
    }

    #[test]
    fn test_core_memory_to_prompt_block() {
        let mut m = CoreMemory::new();
        m.add("name", "alice");
        m.add("lang", "rust");
        let block = m.to_prompt_block();
        assert!(block.contains("## Core Memory"));
        assert!(block.contains("name: alice"));
        assert!(block.contains("lang: rust"));
    }

    #[test]
    fn test_core_memory_apply_changes_add() {
        let mut m = CoreMemory::new();
        m.apply_changes(vec![MemoryChange {
            operation: "add".into(),
            key: "k".into(),
            value: "v".into(),
        }])
        .unwrap();
        assert_eq!(m.get("k").unwrap().value, "v");
    }

    #[test]
    fn test_core_memory_apply_changes_set() {
        let mut m = CoreMemory::new();
        m.add("k", "old");
        m.apply_changes(vec![MemoryChange {
            operation: "set".into(),
            key: "k".into(),
            value: "new".into(),
        }])
        .unwrap();
        assert_eq!(m.get("k").unwrap().value, "new");
    }

    #[test]
    fn test_core_memory_apply_changes_remove() {
        let mut m = CoreMemory::new();
        m.add("k", "v");
        m.apply_changes(vec![MemoryChange {
            operation: "remove".into(),
            key: "k".into(),
            value: String::new(),
        }])
        .unwrap();
        assert!(m.get("k").is_none());
    }

    #[test]
    fn test_core_memory_apply_changes_unknown_op() {
        let mut m = CoreMemory::new();
        let result = m.apply_changes(vec![MemoryChange {
            operation: "unknown".into(),
            key: "k".into(),
            value: "v".into(),
        }]);
        assert!(result.is_err());
    }

    #[test]
    fn test_recall_memory_store_and_search() {
        let mut r = RecallMemory::new();
        r.store("greeting", "hello", "general");
        let results = r.search("hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "greeting");
    }

    #[test]
    fn test_recall_memory_search_missing() {
        let r = RecallMemory::new();
        assert!(r.search("missing").is_empty());
    }

    #[test]
    fn test_recall_memory_all() {
        let mut r = RecallMemory::new();
        r.store("a", "1", "ns1");
        r.store("b", "2", "ns2");
        assert_eq!(r.all().len(), 2);
    }

    #[test]
    fn test_archival_memory_store_and_search() {
        let mut a = ArchivalMemory::new();
        a.store("important document content");
        assert_eq!(a.search("important").len(), 1);
    }

    #[test]
    fn test_archival_memory_search_no_match() {
        let mut a = ArchivalMemory::new();
        a.store("hello world");
        assert!(a.search("missing").is_empty());
    }

    #[test]
    fn test_memory_manager_new() {
        let mm = MemoryManager::new();
        assert_eq!(mm.to_prompt_block(), "");
    }

    #[test]
    fn test_memory_manager_store_core() {
        let mut mm = MemoryManager::new();
        mm.store_core("k", "v");
        assert!(mm.to_prompt_block().contains("k: v"));
    }

    #[test]
    fn test_memory_manager_store_recall() {
        let mut mm = MemoryManager::new();
        mm.store_recall("k", "v", "ns");
        assert!(mm.to_prompt_block().contains("k: v"));
    }

    #[test]
    fn test_memory_manager_search() {
        let mut mm = MemoryManager::new();
        mm.store_recall("key1", "value1", "ns");
        mm.store_archival("archival_value").unwrap();
        let results = mm.search("value1").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_manager_extract_memories() {
        let mut mm = MemoryManager::new();
        let conv = vec![
            ConversationMessage {
                role: "user".into(),
                content: "my name is Alice".into(),
            },
            ConversationMessage {
                role: "user".into(),
                content: "what is the weather".into(),
            },
        ];
        let rt = tokio::runtime::Runtime::new().unwrap();
        let extracted = rt.block_on(mm.extract_memories(&conv)).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].namespace, "extracted");
    }

    #[test]
    fn test_memory_manager_agent_edit_core() {
        let mut mm = MemoryManager::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(mm.agent_edit_core(vec![MemoryChange {
            operation: "add".into(),
            key: "k".into(),
            value: "v".into(),
        }]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_core_memory_default() {
        let m: CoreMemory = Default::default();
        assert!(m.get("anything").is_none());
    }

    #[test]
    fn test_recall_memory_default() {
        let r: RecallMemory = Default::default();
        assert!(r.all().is_empty());
    }
}
