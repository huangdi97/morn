//! Memory module: provides layered memory storage and orchestration.
//!
//! Sub-modules:
//! - `storage` — core types: MemoryRecord and the MemoryLayer trait
//! - `layers` — basic memory layer implementations: WorkingMemory, EpisodicMemory, FlashMemory
//! - `layers_knowledge` — knowledge-oriented layers: SemanticMemory, GraphMemory, and related types
//! - `orchestrator` — experience memory, MemoryHub, and MemoryOrchestrator

use crate::core::error::MornError;
mod layers;
mod layers_knowledge;
mod long_term_experience;
mod orchestrator;
mod storage;

pub use layers::*;
pub use layers_knowledge::*;
pub use long_term_experience::*;
pub use orchestrator::*;
pub use storage::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_working_memory_store_recall() {
        let mut wm = WorkingMemory::default();
        let record = MemoryRecord::new("hello", Value::String("world".to_string()));
        wm.store("hello", record).unwrap();
        let recalled = wm.recall("hello").unwrap().unwrap();
        assert_eq!(recalled.content, "world");
    }

    #[test]
    fn test_working_memory_forget() {
        let mut wm = WorkingMemory::default();
        wm.store("x", MemoryRecord::new("x", Value::Number(1.into())))
            .unwrap();
        wm.forget("x").unwrap();
        assert!(wm.recall("x").unwrap().is_none());
    }

    #[test]
    fn test_working_memory_capacity() {
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
    fn test_episodic_memory() {
        let mut em = EpisodicMemory::default();
        em.store(
            "event_1",
            MemoryRecord::new("event_1", Value::String("started".into())),
        )
        .unwrap();
        em.store(
            "event_2",
            MemoryRecord::new("event_2", Value::String("processing".into())),
        )
        .unwrap();
        assert_eq!(em.size(), 2);
        let recent = em.recent_episodes(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_semantic_memory() {
        let mut sm = SemanticMemory::default();
        sm.store(
            "earth",
            MemoryRecord::new("earth", Value::String("planet".into())),
        )
        .unwrap();
        sm.add_relation("earth", "orbits", "sun");
        let recalled = sm.recall("earth").unwrap().unwrap();
        assert_eq!(recalled.content, "planet");
        let rels = sm.query_relations("earth");
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].predicate, "orbits");
    }

    #[test]
    fn test_experiential_memory() {
        let mut xm = ExperientialMemory::default();
        xm.add_experience("search_failed", Value::String("retry".into()), None);
        xm.add_experience(
            "search_failed",
            Value::String("retry_with_backoff".into()),
            None,
        );
        let top = xm.top_experiences(1);
        assert_eq!(top[0].frequency, 2);
        assert_eq!(xm.size(), 1);
    }

    #[test]
    fn test_graph_memory() {
        let mut gm = GraphMemory::default();
        gm.add_node("n1", "Node1");
        gm.add_node("n2", "Node2");
        gm.add_edge("n1", "n2", "connects_to", 1.0);
        let neighbors = gm.get_neighbors("n1");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].0.id, "n2");
    }

    #[test]
    fn test_flash_memory() {
        let mut fm = FlashMemory::new(10, 3600);
        fm.store(
            "urgent",
            MemoryRecord::new("urgent", Value::String("priority_data".into())).with_priority(10),
        )
        .unwrap();
        let recalled = fm.recall("urgent").unwrap().unwrap();
        assert_eq!(recalled.priority, 10);
    }

    #[test]
    fn test_memory_hub_all_layers() {
        let mut hub = MemoryHub::new();
        assert_eq!(hub.layer_count(), 7);
        hub.store_all(
            "test",
            MemoryRecord::new("test", Value::String("value".into())),
        );
        let results = hub.search_all("test", 10);
        assert_eq!(results.len(), 7);
    }

    #[test]
    fn test_memory_orchestrator() {
        let hub = MemoryHub::new();
        let mut orchestrator = MemoryOrchestrator::new(hub);
        orchestrator.hub_mut().store_all(
            "key",
            MemoryRecord::new("key", Value::String("data".into())),
        );
        let results = orchestrator.decide_with_memory("key").unwrap();
        assert_eq!(results.len(), 7);
    }

    #[test]
    fn test_search_all_layers() {
        let mut hub = MemoryHub::new();
        hub.get_mut("working")
            .unwrap()
            .store(
                "test_key",
                MemoryRecord::new("test_key", Value::String("test_value".into())),
            )
            .unwrap();
        let results = hub.search_all("test_key", 5);
        assert!(results.contains_key("working"));
        assert!(!results["working"].is_empty());
    }

    #[test]
    fn test_compress_all() {
        let mut hub = MemoryHub::new();
        for i in 0..150 {
            hub.store_all(
                &format!("key_{}", i),
                MemoryRecord::new(&format!("key_{}", i), Value::Number(i.into())),
            );
        }
        let compressed = hub.compress_all();
        assert!(compressed.contains_key("working"));
    }

    #[test]
    fn test_experience_compress() {
        let mut xm = ExperientialMemory::new(2);
        xm.add_experience("a", Value::String("1".into()), None);
        xm.add_experience("b", Value::String("2".into()), None);
        xm.add_experience("c", Value::String("3".into()), None);
        let removed = xm.compress().unwrap();
        assert!(removed > 0);
        assert_eq!(xm.size(), 2);
    }

    #[test]
    fn test_graph_traverse() {
        let mut gm = GraphMemory::default();
        gm.add_node("a", "A");
        gm.add_node("b", "B");
        gm.add_node("c", "C");
        gm.add_edge("a", "b", "knows", 1.0);
        gm.add_edge("b", "c", "knows", 1.0);
        let nodes = gm.traverse("a", "knows", 2);
        assert_eq!(nodes.len(), 3);
    }
}
