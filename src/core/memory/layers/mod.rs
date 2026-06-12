//! Memory layers — multi-tier memory system.
pub mod episodic;
pub mod flash;
pub mod working;

pub use episodic::*;
pub use flash::*;
pub use working::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::memory::storage::{MemoryLayer, MemoryRecord};
    use serde_json::Value;

    #[test]
    fn test_working_memory_store_recall() {
        let mut wm = WorkingMemory::new(10);
        let record = MemoryRecord::new("k1", Value::String("v1".into()));
        wm.store("k1", record).unwrap();
        let recalled = wm.recall("k1").unwrap().unwrap();
        assert_eq!(recalled.content, "v1");
    }

    #[test]
    fn test_working_memory_recall_missing() {
        let wm = WorkingMemory::new(10);
        let result = wm.recall("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_working_memory_capacity_eviction() {
        let mut wm = WorkingMemory::new(2);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        wm.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(wm.size(), 2);
    }

    #[test]
    fn test_working_memory_update_existing_within_capacity() {
        let mut wm = WorkingMemory::new(2);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        wm.store("a", MemoryRecord::new("a", Value::Number(99.into())))
            .unwrap();
        assert_eq!(wm.size(), 2);
        let recalled = wm.recall("a").unwrap().unwrap();
        assert_eq!(recalled.content, 99);
    }

    #[test]
    fn test_working_memory_forget() {
        let mut wm = WorkingMemory::new(10);
        wm.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        wm.forget("x").unwrap();
        assert!(wm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_working_memory_compress_no_op_when_under_capacity() {
        let mut wm = WorkingMemory::new(10);
        wm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        wm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        let removed = wm.compress().unwrap();
        assert_eq!(removed, 0);
        assert_eq!(wm.size(), 2);
    }

    #[test]
    fn test_working_memory_search_by_priority() {
        let mut wm = WorkingMemory::new(10);
        wm.store(
            "low",
            MemoryRecord::new("low", Value::String("low".into())).with_priority(1),
        )
        .unwrap();
        wm.store(
            "high",
            MemoryRecord::new("high", Value::String("high".into())).with_priority(10),
        )
        .unwrap();
        let results = wm.search("irrelevant", 2);
        assert_eq!(results.len(), 2);
        assert!(results[0].priority >= results[1].priority);
    }

    #[test]
    fn test_working_memory_context_stack() {
        let mut wm = WorkingMemory::new(10);
        wm.push_context("ctx1");
        wm.push_context("ctx2");
        assert_eq!(wm.pop_context(), Some("ctx2".into()));
        assert_eq!(wm.pop_context(), Some("ctx1".into()));
        assert_eq!(wm.pop_context(), None);
    }

    #[test]
    fn test_episodic_memory_store_recall() {
        let mut em = EpisodicMemory::new(10);
        let record = MemoryRecord::new("evt1", Value::String("started".into()));
        em.store("evt1", record).unwrap();
        let recalled = em.recall("evt1").unwrap().unwrap();
        assert_eq!(recalled.content, "started");
        assert_eq!(
            recalled.metadata.get("episode_type").unwrap(),
            "event_sequence"
        );
    }

    #[test]
    fn test_episodic_memory_recall_missing() {
        let em = EpisodicMemory::new(10);
        assert!(em.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_capacity() {
        let mut em = EpisodicMemory::new(2);
        em.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        em.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        em.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(em.size(), 2);
        assert!(em.recall("a").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_recent_episodes() {
        let mut em = EpisodicMemory::new(10);
        em.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        em.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        let recent = em.recent_episodes(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].key, "b");
    }

    #[test]
    fn test_episodic_memory_forget() {
        let mut em = EpisodicMemory::new(10);
        em.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        em.forget("x").unwrap();
        assert!(em.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_episodic_memory_search() {
        let mut em = EpisodicMemory::new(10);
        em.store(
            "hello_world",
            MemoryRecord::new("hello_world", Value::String("test data".into())),
        )
        .unwrap();
        let results = em.search("hello", 10);
        assert_eq!(results.len(), 1);
        let results = em.search("nonexistent", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_flash_memory_default() {
        let fm = FlashMemory::default();
        assert_eq!(fm.size(), 0);
    }

    #[test]
    fn test_flash_memory_capacity() {
        let mut fm = FlashMemory::new(2, 3600);
        fm.store("a", MemoryRecord::new("a", Value::Number(1.into())))
            .unwrap();
        fm.store("b", MemoryRecord::new("b", Value::Number(2.into())))
            .unwrap();
        fm.store("c", MemoryRecord::new("c", Value::Number(3.into())))
            .unwrap();
        assert_eq!(fm.size(), 2);
    }

    #[test]
    fn test_flash_memory_ttl_applied() {
        let mut fm = FlashMemory::new(10, 60);
        let record = MemoryRecord::new("temp", Value::String("data".into()));
        fm.store("temp", record).unwrap();
        let recalled = fm.recall("temp").unwrap().unwrap();
        assert_eq!(recalled.ttl_secs, Some(60));
    }

    #[test]
    fn test_flash_memory_priority_floor() {
        let mut fm = FlashMemory::new(10, 300);
        let record = MemoryRecord::new("low", Value::String("data".into())).with_priority(1);
        fm.store("low", record).unwrap();
        let recalled = fm.recall("low").unwrap().unwrap();
        assert!(recalled.priority >= 8);
    }

    #[test]
    fn test_flash_memory_high_priority_preserved() {
        let mut fm = FlashMemory::new(10, 300);
        let record = MemoryRecord::new("high", Value::String("data".into())).with_priority(10);
        fm.store("high", record).unwrap();
        let recalled = fm.recall("high").unwrap().unwrap();
        assert_eq!(recalled.priority, 10);
    }

    #[test]
    fn test_flash_memory_recall_missing() {
        let fm = FlashMemory::new(10, 300);
        assert!(fm.recall("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_flash_memory_forget() {
        let mut fm = FlashMemory::new(10, 300);
        fm.store("x", MemoryRecord::new("x", Value::String("y".into())))
            .unwrap();
        fm.forget("x").unwrap();
        assert!(fm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_flash_memory_compress_evicts_expired() {
        let mut fm = FlashMemory::new(10, 0);
        let mut record = MemoryRecord::new("expired", Value::String("gone".into()));
        record.ttl_secs = Some(0);
        fm.store("expired", record).unwrap();
        let removed = fm.compress().unwrap();
        assert_eq!(fm.size(), 0);
    }

    #[test]
    fn test_flash_memory_search() {
        let mut fm = FlashMemory::new(10, 300);
        fm.store(
            "find_me",
            MemoryRecord::new("find_me", Value::String("data".into())),
        )
        .unwrap();
        let results = fm.search("find", 10);
        assert_eq!(results.len(), 1);
    }
}
