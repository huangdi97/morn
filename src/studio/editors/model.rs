#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostTier(pub String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelEditor {
    pub name: String,
    pub provider: String,
    pub model_name: String,
    pub parameters: serde_json::Value,
    pub fallback: Option<String>,
    pub cost_tier: CostTier,
}

impl ModelEditor {
    pub fn new(name: &str) -> Self {
        ModelEditor {
            name: name.to_string(),
            provider: "deepseek".to_string(),
            model_name: "deepseek-chat".to_string(),
            parameters: serde_json::json!({"temperature": 0.7, "max_tokens": 2048}),
            fallback: None,
            cost_tier: CostTier("low".to_string()),
        }
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "model",
            "name": self.name,
            "provider": self.provider,
            "model_name": self.model_name,
            "parameters": self.parameters,
            "fallback": self.fallback,
            "cost_tier": self.cost_tier.0,
        })
    }
}
