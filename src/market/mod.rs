//! market — Marketplace listings, billing, payments, and creator revenue.
use crate::core::error::MornError;
pub mod billing;
pub mod gateway;
pub mod gateway_mock;
pub mod gateway_stripe;
pub mod marketplace;
pub mod revenue;

pub use marketplace::AgentVersion;
pub use marketplace::License;
pub use marketplace::Listing;
pub use marketplace::Marketplace;
pub use marketplace::Review;
pub use marketplace::Transaction;

pub fn render_market_browser() -> String {
    let storage = match crate::core::storage::Storage::new_in_memory() {
        Ok(storage) => storage,
        Err(err) => return format!("Market unavailable: {}", err),
    };
    let market = Marketplace::new(storage);
    let listings = market.list(None);
    let mut output = String::from("Market Browser\n");

    for listing in listings {
        output.push_str(&format!(
            "{} | {} | {} | ¥{:.3} | {:.1}★ | {} downloads\n",
            listing.id,
            listing.item_type,
            listing.name,
            listing.price,
            listing.rating,
            listing.downloads
        ));
    }

    output
}

pub fn render_listing_detail(id: &str) -> String {
    let storage = match crate::core::storage::Storage::new_in_memory() {
        Ok(storage) => storage,
        Err(err) => return format!("Market unavailable: {}", err),
    };
    let market = Marketplace::new(storage);

    match market.get(id) {
        Some(listing) => format!(
            "Listing Detail\nid: {}\ntype: {}\nname: {}\nauthor: {}\nprice: ¥{:.3}\nrating: {:.1}\ndownloads: {}\ndescription: {}",
            listing.id,
            listing.item_type,
            listing.name,
            listing.author,
            listing.price,
            listing.rating,
            listing.downloads,
            listing.description
        ),
        None => format!("Listing not found: {}", id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn marketplace_initializes_builtin_listings() {
        let market = Marketplace::new(Storage::new_in_memory().unwrap());

        assert_eq!(market.list(None).len(), 7);
    }

    #[test]
    fn category_query_filters_builtin_type() {
        let market = Marketplace::new(Storage::new_in_memory().unwrap());

        let personas = market.list(Some("persona"));

        assert_eq!(personas.len(), 1);
        assert_eq!(personas[0].item_type, "persona");
    }

    #[test]
    fn module_reexports_license_listing_and_transaction_types() {
        let listing = Listing {
            id: "listing-1".into(),
            item_type: "tool".into(),
            name: "Tool".into(),
            description: "desc".into(),
            price: 0.0,
            author: "tester".into(),
            rating: 0.0,
            downloads: 0,
            created_at: "now".into(),
        };
        let transaction = Transaction {
            id: "tx-1".into(),
            listing_id: listing.id.clone(),
            buyer: "user-1".into(),
            amount: listing.price,
            timestamp: "now".into(),
        };
        let license = License {
            id: "lic-1".into(),
            listing_id: listing.id.clone(),
            user_id: transaction.buyer.clone(),
            granted_at: "now".into(),
            expires_at: None,
        };

        assert_eq!(license.listing_id, transaction.listing_id);
    }

    #[test]
    fn render_market_browser_lists_builtin_items() {
        let output = render_market_browser();

        assert!(output.contains("Market Browser"));
        assert!(output.contains("listing-tool-web-search"));
        assert!(output.contains("Web Search Pro"));
    }

    #[test]
    fn render_listing_detail_shows_builtin_detail() {
        let output = render_listing_detail("listing-tool-web-search");

        assert!(output.contains("Listing Detail"));
        assert!(output.contains("Web Search Pro"));
        assert!(output.contains("Advanced web search"));
    }

    #[test]
    fn render_listing_detail_reports_missing_listing() {
        let output = render_listing_detail("missing");

        assert_eq!(output, "Listing not found: missing");
    }
}
