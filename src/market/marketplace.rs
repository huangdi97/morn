use crate::core::registry::{Capability, Registry};
use crate::core::storage::Storage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Listing {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub author: String,
    pub rating: f64,
    pub downloads: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub id: String,
    pub listing_id: String,
    pub buyer: String,
    pub amount: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct License {
    pub id: String,
    pub listing_id: String,
    pub user_id: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
}

pub struct Marketplace {
    storage: Storage,
}

impl Marketplace {
    pub fn new(storage: Storage) -> Self {
        let market = Marketplace { storage };
        market.list_builtin();
        market
    }

    fn list_builtin(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let builtin = vec![
            Listing {
                id: "listing-tool-web-search".into(),
                item_type: "tool".into(),
                name: "Web Search Pro".into(),
                description: "Advanced web search with multi-source aggregation".into(),
                price: 0.001,
                author: "Morn Labs".into(),
                rating: 4.5,
                downloads: 1230,
                created_at: now.clone(),
            },
            Listing {
                id: "listing-knowledge-stocks".into(),
                item_type: "knowledge".into(),
                name: "Stock Market Data".into(),
                description: "Real-time stock quotes and historical data".into(),
                price: 0.01,
                author: "Morn Labs".into(),
                rating: 4.2,
                downloads: 890,
                created_at: now.clone(),
            },
            Listing {
                id: "listing-skill-research".into(),
                item_type: "skill".into(),
                name: "Deep Research Skill".into(),
                description: "Multi-step research with cross-verification".into(),
                price: 0.01,
                author: "Morn Labs".into(),
                rating: 4.8,
                downloads: 560,
                created_at: now.clone(),
            },
            Listing {
                id: "listing-persona-analyst".into(),
                item_type: "persona".into(),
                name: "Financial Analyst".into(),
                description: "Data-driven financial analysis persona".into(),
                price: 0.0,
                author: "Morn Labs".into(),
                rating: 4.3,
                downloads: 2100,
                created_at: now.clone(),
            },
            Listing {
                id: "listing-agent-research".into(),
                item_type: "agent".into(),
                name: "Research Agent".into(),
                description: "Full-featured research agent with web search + analysis".into(),
                price: 0.05,
                author: "Morn Labs".into(),
                rating: 4.6,
                downloads: 340,
                created_at: now.clone(),
            },
            Listing {
                id: "listing-workflow-report".into(),
                item_type: "workflow".into(),
                name: "Weekly Report Generator".into(),
                description: "Automated weekly report generation workflow".into(),
                price: 0.03,
                author: "Morn Labs".into(),
                rating: 4.1,
                downloads: 120,
                created_at: now.clone(),
            },
        ];
        for listing in builtin {
            if self
                .storage
                .get_listing(&listing.id)
                .ok()
                .flatten()
                .is_none()
            {
                let _ = self.storage.save_listing(&listing);
            }
        }
    }

    pub fn list(&self, filter: Option<&str>) -> Vec<Listing> {
        self.storage.list_listings(filter).unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<Listing> {
        self.storage.get_listing(id).ok().flatten()
    }

    pub fn purchase(&self, listing_id: &str, user_id: &str) -> Result<License, String> {
        let listing = self
            .storage
            .get_listing(listing_id)?
            .ok_or("Listing not found")?;

        let tx = Transaction {
            id: format!("tx-{}", uuid::Uuid::new_v4()),
            listing_id: listing_id.to_string(),
            buyer: user_id.to_string(),
            amount: listing.price,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.save_transaction(&tx)?;

        let license = License {
            id: format!("lic-{}", uuid::Uuid::new_v4()),
            listing_id: listing_id.to_string(),
            user_id: user_id.to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
            expires_at: if listing.price > 0.0 {
                Some((chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339())
            } else {
                None
            },
        };
        self.storage.save_license(&license)?;
        Ok(license)
    }

    pub fn publish(&self, listing: Listing) -> Result<(), String> {
        if self.storage.get_listing(&listing.id)?.is_some() {
            return Err("Listing already exists".to_string());
        }
        self.storage.save_listing(&listing)
    }

    pub fn install(&self, listing_id: &str, user_id: &str) -> Result<(), String> {
        self.storage
            .get_listing(listing_id)?
            .ok_or("Listing not found")?;
        let user_licenses = self.storage.get_user_licenses(user_id)?;
        if !user_licenses.iter().any(|l| l.listing_id == listing_id) {
            return Err("User has not purchased this listing".to_string());
        }
        Ok(())
    }

    pub fn rate(
        &self,
        listing_id: &str,
        _user_id: &str,
        score: u8,
        _review: &str,
    ) -> Result<(), String> {
        let listing = self
            .storage
            .get_listing(listing_id)?
            .ok_or("Listing not found")?;
        let clamped_score = score.min(5) as f64;
        let new_rating = (listing.rating * listing.downloads as f64 + clamped_score)
            / (listing.downloads as f64 + 1.0);
        let new_downloads = listing.downloads + 1;
        self.storage
            .update_listing_rating(listing_id, new_rating, new_downloads)
    }

    pub fn search(&self, query: &str) -> Vec<Listing> {
        let q = query.to_lowercase();
        self.storage
            .list_listings(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|l| {
                l.name.to_lowercase().contains(&q)
                    || l.description.to_lowercase().contains(&q)
                    || l.tags().iter().any(|t| t.contains(&q))
            })
            .collect()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.storage.list_listings(None).unwrap_or_default();
        vec![]
    }

    pub fn user_licenses(&self, user_id: &str) -> Vec<License> {
        self.storage.get_user_licenses(user_id).unwrap_or_default()
    }

    pub fn install_to_registry(
        &self,
        listing_id: &str,
        registry: &mut Registry,
    ) -> Result<(), String> {
        let listing = self
            .storage
            .get_listing(listing_id)?
            .ok_or("Listing not found")?;
        let cap = Capability {
            id: format!("market-{}", listing.id),
            name: listing.name,
            domain: listing.item_type,
            actions: vec![listing.id.clone()],
            description: listing.description,
            trust_score: listing.rating * 20.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
        };
        registry.register(cap);
        Ok(())
    }
}

impl Listing {
    pub fn tags(&self) -> Vec<String> {
        vec![self.item_type.clone(), self.name.to_lowercase()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> Storage {
        Storage::new_in_memory().unwrap()
    }

    #[test]
    fn test_list_marketplace() {
        let market = Marketplace::new(test_storage());
        let all = market.list(None);
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_filter_by_type() {
        let market = Marketplace::new(test_storage());
        let tools = market.list(Some("tool"));
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "Web Search Pro");
    }

    #[test]
    fn test_purchase_and_install() {
        let market = Marketplace::new(test_storage());
        let license = market
            .purchase("listing-tool-web-search", "user-1")
            .unwrap();
        assert_eq!(license.listing_id, "listing-tool-web-search");
        assert!(market.install("listing-tool-web-search", "user-1").is_ok());
    }

    #[test]
    fn test_publish() {
        let market = Marketplace::new(test_storage());
        let listing = Listing {
            id: "listing-test-1".into(),
            item_type: "tool".into(),
            name: "Test Tool".into(),
            description: "test".into(),
            price: 0.0,
            author: "test".into(),
            rating: 0.0,
            downloads: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        market.publish(listing).unwrap();
        assert_eq!(market.list(None).len(), 7);
    }

    #[test]
    fn test_rating() {
        let market = Marketplace::new(test_storage());
        market
            .rate("listing-tool-web-search", "user-1", 5, "Great!")
            .unwrap();
        let listing = market.get("listing-tool-web-search").unwrap();
        assert!(listing.rating > 4.5);
    }

    #[test]
    fn test_search() {
        let market = Marketplace::new(test_storage());
        let results = market.search("search");
        assert!(!results.is_empty());
        assert!(results.iter().any(|l| l.name.contains("Search")));
    }

    #[test]
    fn test_purchase_without_license() {
        let market = Marketplace::new(test_storage());
        assert!(market
            .install("listing-tool-web-search", "unknown-user")
            .is_err());
    }

    #[test]
    fn test_install_to_registry() {
        let mut registry = Registry::new(None, None);
        let market = Marketplace::new(test_storage());
        market
            .purchase("listing-tool-web-search", "user-1")
            .unwrap();
        market
            .install_to_registry("listing-tool-web-search", &mut registry)
            .unwrap();
        let cap = registry.get("market-listing-tool-web-search");
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().name, "Web Search Pro");
    }
}
