//! model — Defines model configuration and model invocation components.
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[derive(Debug, Clone)]
pub enum CostTier {
    Free,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct ModelParameters {
    pub temperature: f64,
    pub max_tokens: u32,
    pub top_p: f64,
}

impl Default for ModelParameters {
    fn default() -> Self {
        ModelParameters {
            temperature: 0.6,
            max_tokens: 4096,
            top_p: 0.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub id: String,
    pub provider: String,
    pub model_name: String,
    pub base_url: String,
    pub api_key: String,
    pub parameters: ModelParameters,
    pub fallback: Option<String>,
    pub cost_tier: CostTier,
}

pub struct ModelComponent {
    id: String,
    config: ModelConfig,
}

impl ModelComponent {
    pub fn new(config: ModelConfig) -> Self {
        ModelComponent {
            id: config.id.clone(),
            config,
        }
    }

    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    pub fn chat(&self, _prompt: &str, _system: &str) -> Result<String, String> {
        let model = &self.config.model_name;
        let provider = &self.config.provider;
        Ok(format!("[{}:{}] (simulated response)", provider, model))
    }
}

impl Component for ModelComponent {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "model"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for ModelComponent {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "prompt".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "response".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for ModelComponent {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

pub fn create_default_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig {
            id: "model-deepseek-chat".into(),
            provider: "deepseek".into(),
            model_name: "deepseek-chat".into(),
            base_url: "https://api.deepseek.com".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: Some("model-deepseek-reasoner".into()),
            cost_tier: CostTier::Low,
        },
        ModelConfig {
            id: "model-deepseek-reasoner".into(),
            provider: "deepseek".into(),
            model_name: "deepseek-reasoner".into(),
            base_url: "https://api.deepseek.com".into(),
            api_key: String::new(),
            parameters: ModelParameters {
                temperature: 0.3,
                max_tokens: 8192,
                top_p: 0.9,
            },
            fallback: None,
            cost_tier: CostTier::Medium,
        },
    ]
}

pub fn create_model_component(config: ModelConfig) -> ModelComponent {
    ModelComponent::new(config)
}
