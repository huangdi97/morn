//! storage — Defines serializable memory entries and storage records.
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

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
