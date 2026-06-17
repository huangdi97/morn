//! memory — Defines memory components used to persist conversational context.
use crate::core::component::IOComponent;
use crate::core::error::MornError;

pub mod mdrm;
mod simple;
mod storage;

pub use mdrm::{CausalChain, Entity as MdrEntity, MDRMGraph, Relation as MdrRelation};
pub use simple::{create_default_memory, SqliteMemory};
pub use storage::{
    ArchivalMemory, ConversationMessage, CoreMemory, MemoryChange, MemoryItem, MemoryManager,
    RecallMemory,
};

pub trait Memory: IOComponent {
    fn store(&mut self, key: &str, value: &str, namespace: &str) -> Result<(), MornError>;
    fn retrieve(&self, key: &str, namespace: &str) -> Result<Option<String>, MornError>;
    fn search(&self, query: &str, namespace: &str) -> Result<Vec<(String, String)>, MornError>;
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
