//! model — Model editor with provider, parameters, fallback, and cost tier configuration.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostTier(pub String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelParameters {
    pub temperature: f64,
    pub max_tokens: u64,
    pub top_p: f64,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
}

impl Default for ModelParameters {
    fn default() -> Self {
        ModelParameters {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
}

impl ModelParameters {
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::json!({
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "top_p": self.top_p,
            "frequency_penalty": self.frequency_penalty,
            "presence_penalty": self.presence_penalty,
        })
    }
}

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
        let default_parameters = ModelParameters::default();
        ModelEditor {
            name: name.to_string(),
            provider: "deepseek".to_string(),
            model_name: "deepseek-chat".to_string(),
            parameters: default_parameters.to_value(),
            fallback: None,
            cost_tier: CostTier("low".to_string()),
        }
    }

    pub fn set_parameter(&mut self, key: &str, value: serde_json::Value) {
        if !self.parameters.is_object() {
            self.parameters = serde_json::json!({});
        }
        if let Some(params) = self.parameters.as_object_mut() {
            params.insert(key.to_string(), value);
        }
    }

    pub fn apply_parameters(&mut self, parameters: ModelParameters) {
        self.parameters = parameters.to_value();
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
