use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatorEarnings {
    pub creator_id: String,
    pub total_earnings: f64,
    pub pending_payout: f64,
    pub sale_count: u64,
}

impl CreatorEarnings {
    pub fn new(creator_id: &str) -> Self {
        CreatorEarnings {
            creator_id: creator_id.to_string(),
            total_earnings: 0.0,
            pending_payout: 0.0,
            sale_count: 0,
        }
    }

    pub fn record_sale(&mut self, amount: f64, platform_cut: f64) {
        let creator_share = amount * (1.0 - platform_cut);
        self.total_earnings += creator_share;
        self.pending_payout += creator_share;
        self.sale_count += 1;
    }

    pub fn request_payout(&mut self) -> f64 {
        let payout = self.pending_payout;
        self.pending_payout = 0.0;
        payout
    }

    pub fn summary(&self) -> String {
        format!(
            "Creator {}: {:.2} total, {:.2} pending, {} sales",
            self.creator_id, self.total_earnings, self.pending_payout, self.sale_count
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevenueManager {
    pub creators: HashMap<String, CreatorEarnings>,
    pub platform_cut: f64,
}

impl RevenueManager {
    pub fn new(platform_cut: f64) -> Self {
        RevenueManager {
            creators: HashMap::new(),
            platform_cut,
        }
    }

    pub fn register_creator(&mut self, creator_id: &str) {
        self.creators
            .entry(creator_id.to_string())
            .or_insert_with(|| CreatorEarnings::new(creator_id));
    }

    pub fn record_sale(&mut self, creator_id: &str, amount: f64) {
        let cut = self.platform_cut;
        self.creators
            .entry(creator_id.to_string())
            .or_insert_with(|| CreatorEarnings::new(creator_id))
            .record_sale(amount, cut);
    }

    pub fn get_earnings(&self, creator_id: &str) -> Option<&CreatorEarnings> {
        self.creators.get(creator_id)
    }

    pub fn all_earnings(&self) -> Vec<&CreatorEarnings> {
        let mut result: Vec<&CreatorEarnings> = self.creators.values().collect();
        result.sort_by_key(|e| e.creator_id.clone());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creator_earnings_record_sale_and_payout() {
        let mut earnings = CreatorEarnings::new("creator-1");
        earnings.record_sale(100.0, 0.1);
        assert_eq!(earnings.total_earnings, 90.0);
        assert_eq!(earnings.pending_payout, 90.0);
        assert_eq!(earnings.sale_count, 1);

        let payout = earnings.request_payout();
        assert_eq!(payout, 90.0);
        assert_eq!(earnings.pending_payout, 0.0);
    }

    #[test]
    fn revenue_manager_tracks_multiple_creators() {
        let mut mgr = RevenueManager::new(0.15);
        mgr.register_creator("alice");
        mgr.register_creator("bob");

        mgr.record_sale("alice", 200.0);
        mgr.record_sale("bob", 100.0);
        mgr.record_sale("alice", 50.0);

        let alice = mgr.get_earnings("alice").unwrap();
        assert!((alice.total_earnings - (250.0 * 0.85)).abs() < 1e-9);
        assert_eq!(alice.sale_count, 2);

        let bob = mgr.get_earnings("bob").unwrap();
        assert!((bob.total_earnings - (100.0 * 0.85)).abs() < 1e-9);
        assert_eq!(bob.sale_count, 1);
    }

    #[test]
    fn revenue_manager_auto_registers_on_sale() {
        let mut mgr = RevenueManager::new(0.1);
        mgr.record_sale("charlie", 80.0);

        let earnings = mgr.get_earnings("charlie").unwrap();
        assert!((earnings.total_earnings - 72.0).abs() < 1e-9);
        assert_eq!(earnings.sale_count, 1);

        let all = mgr.all_earnings();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].creator_id, "charlie");
    }
}
