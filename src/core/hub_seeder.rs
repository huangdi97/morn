//! Seeds the marketplace hub with initial workflow templates and agent templates
//! during first-time setup or when listings are missing.

use crate::core::agent_templates::AGENT_TEMPLATES;
use crate::core::storage::Storage;
use crate::core::workflow::WorkflowTemplate;
use crate::market::{Listing, Marketplace};

/// Publishes built-in workflow templates and agent templates to the marketplace.
///
/// For each built-in [`WorkflowTemplate`] and each entry in [`AGENT_TEMPLATES`],
/// a free [`Listing`] is published under the `"Morn Labs"` author. Existing
/// listings are skipped (identified by a deterministic `id` prefix), making this
/// function safe to call repeatedly.
///
/// This is a no-op when `storage` is `None`.
pub fn seed_hub_data(storage: &Option<Storage>) {
    let storage = match storage {
        Some(s) => s,
        None => return,
    };
    let market = Marketplace::new(storage.clone());

    for template in WorkflowTemplate::list_builtin() {
        let id = format!("listing-workflow-{}", template.id);
        if market.get(&id).is_some() {
            continue;
        }
        let _ = market.publish(Listing {
            id,
            item_type: "workflow".into(),
            name: template.name,
            description: template.description,
            price: 0.0,
            author: "Morn Labs".into(),
            rating: 0.0,
            downloads: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    for agent in AGENT_TEMPLATES.iter() {
        let id = format!("listing-agent-template-{}", agent.name);
        if market.get(&id).is_some() {
            continue;
        }
        let _ = market.publish(Listing {
            id,
            item_type: "agent".into(),
            name: agent.name.to_string(),
            description: agent.description.to_string(),
            price: 0.0,
            author: "Morn Labs".into(),
            rating: 0.0,
            downloads: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn test_seed_hub_data_populates_listings() {
        let storage = Storage::new_in_memory().unwrap();
        seed_hub_data(&Some(storage.clone()));
        let market = Marketplace::new(storage.clone());
        let count = market.list(None).len();
        assert!(count > 0);
    }

    #[test]
    fn test_seed_hub_data_idempotent() {
        let storage = Storage::new_in_memory().unwrap();
        seed_hub_data(&Some(storage.clone()));
        let market = Marketplace::new(storage.clone());
        let count = market.list(None).len();
        seed_hub_data(&Some(storage.clone()));
        let count2 = market.list(None).len();
        assert_eq!(count, count2);
    }
}
