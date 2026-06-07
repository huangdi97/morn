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
