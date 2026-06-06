use crate::core::security::SecurityGuard;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GovernanceConfig {
    pub policy_exceptions: Vec<PolicyException>,
    pub api_keys: Vec<ApiKeyInfo>,
    pub channel_bindings: Vec<ChannelBinding>,
    pub trust_threshold: f64,
    pub approval_queue: Vec<ApprovalItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyException {
    pub id: String,
    pub policy_name: String,
    pub reason: String,
    pub expires_at: String,
    pub created_by: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub masked_key: String,
    pub created_at: String,
    pub last_used: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelBinding {
    pub channel: String,
    pub webhook_url: String,
    pub enabled: bool,
    pub last_active: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApprovalItem {
    pub id: String,
    pub action: String,
    pub requester: String,
    pub reason: String,
    pub requested_at: String,
}

pub struct Governance {
    security: Option<SecurityGuard>,
    exceptions: Vec<PolicyException>,
    api_keys: Vec<ApiKeyInfo>,
    bindings: Vec<ChannelBinding>,
    trust_threshold: f64,
    approval_queue: Vec<ApprovalItem>,
}

impl Governance {
    pub fn new(security: Option<SecurityGuard>) -> Self {
        Governance {
            security,
            exceptions: vec![],
            api_keys: vec![ApiKeyInfo {
                id: "key-1".into(),
                name: "DeepSeek Production".into(),
                provider: "deepseek".into(),
                masked_key: "sk-****-abcd".into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_used: chrono::Utc::now().to_rfc3339(),
            }],
            bindings: vec![ChannelBinding {
                channel: "cli".into(),
                webhook_url: "builtin".into(),
                enabled: true,
                last_active: chrono::Utc::now().to_rfc3339(),
            }],
            trust_threshold: 50.0,
            approval_queue: vec![],
        }
    }

    pub fn get_config(&self) -> GovernanceConfig {
        GovernanceConfig {
            policy_exceptions: self.exceptions.clone(),
            api_keys: self.api_keys.clone(),
            channel_bindings: self.bindings.clone(),
            trust_threshold: self.trust_threshold,
            approval_queue: self.approval_queue.clone(),
        }
    }

    pub fn add_exception(&mut self, exception: PolicyException) {
        self.exceptions.push(exception);
    }

    pub fn remove_exception(&mut self, id: &str) {
        self.exceptions.retain(|e| e.id != id);
    }

    pub fn add_api_key(&mut self, key: ApiKeyInfo) {
        self.api_keys.push(key);
    }

    pub fn remove_api_key(&mut self, id: &str) {
        self.api_keys.retain(|k| k.id != id);
    }

    pub fn set_trust_threshold(&mut self, threshold: f64) {
        self.trust_threshold = threshold.clamp(0.0, 100.0);
    }

    pub fn approve_item(&mut self, id: &str) -> Result<(), String> {
        let idx = self
            .approval_queue
            .iter()
            .position(|a| a.id == id)
            .ok_or("Approval item not found")?;
        self.approval_queue.remove(idx);
        Ok(())
    }

    pub fn reject_item(&mut self, id: &str) -> Result<(), String> {
        let idx = self
            .approval_queue
            .iter()
            .position(|a| a.id == id)
            .ok_or("Approval item not found")?;
        self.approval_queue.remove(idx);
        Ok(())
    }

    pub fn list_policies(&self) -> Vec<String> {
        self.security
            .as_ref()
            .map(|s| {
                s.list_policies()
                    .iter()
                    .map(|p| format!("{} [{}]", p.name, p.level.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_governance() -> Governance {
        Governance::new(None)
    }

    #[test]
    fn test_new_has_default_api_key() {
        let g = make_governance();
        let config = g.get_config();
        assert_eq!(config.api_keys.len(), 1);
        assert_eq!(config.api_keys[0].name, "DeepSeek Production");
    }

    #[test]
    fn test_new_has_default_channel_binding() {
        let g = make_governance();
        let config = g.get_config();
        assert_eq!(config.channel_bindings.len(), 1);
        assert_eq!(config.channel_bindings[0].channel, "cli");
    }

    #[test]
    fn test_default_trust_threshold() {
        let g = make_governance();
        let config = g.get_config();
        assert_eq!(config.trust_threshold, 50.0);
    }

    #[test]
    fn test_add_exception() {
        let mut g = make_governance();
        let exc = PolicyException {
            id: "exc-1".into(),
            policy_name: "test-policy".into(),
            reason: "testing".into(),
            expires_at: "2026-01-01".into(),
            created_by: "tester".into(),
        };
        g.add_exception(exc);
        let config = g.get_config();
        assert_eq!(config.policy_exceptions.len(), 1);
        assert_eq!(config.policy_exceptions[0].id, "exc-1");
    }

    #[test]
    fn test_remove_exception() {
        let mut g = make_governance();
        let exc = PolicyException {
            id: "exc-1".into(),
            policy_name: "test".into(),
            reason: "test".into(),
            expires_at: "2026-01-01".into(),
            created_by: "tester".into(),
        };
        g.add_exception(exc);
        g.remove_exception("exc-1");
        let config = g.get_config();
        assert!(config.policy_exceptions.is_empty());
    }

    #[test]
    fn test_add_api_key() {
        let mut g = make_governance();
        let key = ApiKeyInfo {
            id: "key-2".into(),
            name: "Test Key".into(),
            provider: "openai".into(),
            masked_key: "sk-****-test".into(),
            created_at: "now".into(),
            last_used: "now".into(),
        };
        g.add_api_key(key);
        let config = g.get_config();
        assert_eq!(config.api_keys.len(), 2);
    }

    #[test]
    fn test_remove_api_key() {
        let mut g = make_governance();
        g.remove_api_key("key-1");
        let config = g.get_config();
        assert!(config.api_keys.is_empty());
    }

    #[test]
    fn test_set_trust_threshold_clamps_low() {
        let mut g = make_governance();
        g.set_trust_threshold(-10.0);
        assert_eq!(g.get_config().trust_threshold, 0.0);
    }

    #[test]
    fn test_set_trust_threshold_clamps_high() {
        let mut g = make_governance();
        g.set_trust_threshold(150.0);
        assert_eq!(g.get_config().trust_threshold, 100.0);
    }

    #[test]
    fn test_set_trust_threshold_normal() {
        let mut g = make_governance();
        g.set_trust_threshold(75.5);
        assert_eq!(g.get_config().trust_threshold, 75.5);
    }

    #[test]
    fn test_approve_item_removes_from_queue() {
        let mut g = make_governance();
        let item = ApprovalItem {
            id: "approve-1".into(),
            action: "delete".into(),
            requester: "user".into(),
            reason: "test".into(),
            requested_at: "now".into(),
        };
        g.approval_queue.push(item);
        assert_eq!(g.get_config().approval_queue.len(), 1);

        let result = g.approve_item("approve-1");
        assert!(result.is_ok());
        assert!(g.get_config().approval_queue.is_empty());
    }

    #[test]
    fn test_approve_item_not_found() {
        let mut g = make_governance();
        let result = g.approve_item("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_item_removes_from_queue() {
        let mut g = make_governance();
        let item = ApprovalItem {
            id: "reject-1".into(),
            action: "delete".into(),
            requester: "user".into(),
            reason: "test".into(),
            requested_at: "now".into(),
        };
        g.approval_queue.push(item);

        let result = g.reject_item("reject-1");
        assert!(result.is_ok());
        assert!(g.get_config().approval_queue.is_empty());
    }

    #[test]
    fn test_reject_item_not_found() {
        let mut g = make_governance();
        let result = g.reject_item("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_policies_without_security() {
        let g = make_governance();
        let policies = g.list_policies();
        assert!(policies.is_empty());
    }
}
