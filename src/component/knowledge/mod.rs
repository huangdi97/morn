//! Knowledge module — shared types, trait, and module aggregator.

use std::collections::HashMap;

use crate::core::component::IOComponent;

mod fulltext;
mod persistent;
mod structured;
mod vector;

pub use fulltext::FulltextKnowledge;
pub use persistent::{FileKnowledge, SqliteKnowledge, StaticKnowledge};
pub use structured::StructuredKnowledge;
pub use vector::VectorKnowledge;

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub key: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UpdateStrategy {
    Manual,
    Periodic { interval_secs: u64 },
    OnDemand,
}

pub trait Knowledge: IOComponent {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String>;
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String>;
}

pub fn create_default_knowledge() -> Vec<Box<dyn Knowledge>> {
    let mut static_data = HashMap::new();
    static_data.insert("stock_code_AAPL".into(), "AAPL".into());
    static_data.insert(
        "api_endpoint_deepseek".into(),
        "https://api.deepseek.com".into(),
    );

    vec![Box::new(StaticKnowledge::new(static_data))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_strategy_default() {
        assert_eq!(UpdateStrategy::Manual, UpdateStrategy::Manual);
    }

    #[test]
    fn test_update_strategy_periodic() {
        let s = UpdateStrategy::Periodic {
            interval_secs: 3600,
        };
        match s {
            UpdateStrategy::Periodic { interval_secs } => assert_eq!(interval_secs, 3600),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_update_strategy_on_demand() {
        assert_eq!(UpdateStrategy::OnDemand, UpdateStrategy::OnDemand);
    }

    #[test]
    fn test_update_strategy_serialization() {
        let manual = serde_json::to_string(&UpdateStrategy::Manual).unwrap();
        let periodic =
            serde_json::to_string(&UpdateStrategy::Periodic { interval_secs: 60 }).unwrap();
        assert!(manual.contains("Manual"));
        assert!(periodic.contains("Periodic"));
        assert!(periodic.contains("60"));
    }

    #[test]
    fn test_create_default_knowledge() {
        let knowledge = create_default_knowledge();
        assert_eq!(knowledge.len(), 1);
    }
}
