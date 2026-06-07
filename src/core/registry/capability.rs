//! capability — Defines registered agent capabilities and their usage metrics.
#[allow(dead_code)] /* 预留：能力市场和路由统计字段 */
#[derive(Clone)]
pub struct Capability {
    pub id: String,
    pub version: String,
    pub name: String,
    pub domain: String,
    pub actions: Vec<String>,
    pub description: String,
    pub trust_score: f64,
    pub total_calls: u64,
    pub success_calls: u64,
    pub avg_latency_ms: f64,
    pub visibility: String,
    pub owner_id: Option<String>,
    pub team_id: Option<String>,
    pub daily_quota: u64,
}

impl Capability {
    pub(super) fn default_chat_agent() -> Self {
        Self {
            id: "chat-agent".to_string(),
            version: "0.1.0".to_string(),
            name: "Chat Agent".to_string(),
            domain: "general".to_string(),
            actions: vec![
                "chat".to_string(),
                "analyze".to_string(),
                "report".to_string(),
            ],
            description: "General purpose chat agent powered by LLM".to_string(),
            trust_score: 70.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "public".to_string(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        }
    }

    fn record_call(&mut self, success: bool, latency_ms: f64) {
        self.total_calls += 1;
        if success {
            self.success_calls += 1;
        }

        let execution_success = if self.total_calls > 0 {
            self.success_calls as f64 / self.total_calls as f64
        } else {
            0.0
        };

        let latency_score = if latency_ms > 0.0 {
            (1000.0 / latency_ms).min(1.0)
        } else {
            0.0
        };

        self.avg_latency_ms = if self.total_calls > 1 {
            (self.avg_latency_ms * (self.total_calls as f64 - 1.0) + latency_ms)
                / self.total_calls as f64
        } else {
            latency_ms
        };

        self.trust_score =
            70.0 * 0.3 + execution_success * 30.0 + latency_score * 20.0 + 50.0 * 0.2;
    }
}

impl super::Registry {
    /// Updates capability usage statistics for an id using success status and latency in milliseconds.
    pub fn update_trust_score(&mut self, id: &str, success: bool, latency_ms: f64) {
        if let Some(cap) = self.capabilities.get_mut(id) {
            cap.record_call(success, latency_ms);
        }
    }
}
