//! model — Defines model configuration and model invocation components.
use crate::core::error::MornError;
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

    pub fn chat(&self, _prompt: &str, _system: &str) -> Result<String, MornError> {
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
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
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
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), MornError> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, MornError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parameters_default() {
        let p = ModelParameters::default();
        assert_eq!(p.temperature, 0.6);
        assert_eq!(p.max_tokens, 4096);
        assert_eq!(p.top_p, 0.9);
    }

    #[test]
    fn test_model_component_chat_returns_simulated_response() {
        let config = ModelConfig {
            id: "test-model".into(),
            provider: "test".into(),
            model_name: "test-model".into(),
            base_url: "http://localhost".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Free,
        };
        let component = ModelComponent::new(config);
        let result = component.chat("hello", "system").unwrap();
        assert_eq!(result, "[test:test-model] (simulated response)");
    }

    #[test]
    fn test_model_component_chat_with_empty_key() {
        let config = ModelConfig {
            id: "no-key-model".into(),
            provider: "test".into(),
            model_name: "test".into(),
            base_url: "http://localhost".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Free,
        };
        let component = ModelComponent::new(config);
        let result = component.chat("prompt", "system").unwrap();
        assert!(result.contains("simulated"));
    }

    #[test]
    fn test_create_default_models_count() {
        let models = create_default_models();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_create_default_models_fields() {
        let models = create_default_models();
        assert_eq!(models[0].id, "model-deepseek-chat");
        assert_eq!(models[0].provider, "deepseek");
        assert_eq!(models[0].model_name, "deepseek-chat");
        assert_eq!(models[0].fallback, Some("model-deepseek-reasoner".into()));
        assert_eq!(models[1].id, "model-deepseek-reasoner");
        assert_eq!(models[1].model_name, "deepseek-reasoner");
        assert!(models[1].fallback.is_none());
    }

    #[test]
    fn test_model_component_component_trait() {
        let config = ModelConfig {
            id: "comp-test".into(),
            provider: "p".into(),
            model_name: "m".into(),
            base_url: "http://localhost".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Low,
        };
        let mut component = ModelComponent::new(config);
        assert_eq!(component.id(), "comp-test");
        assert_eq!(component.type_name(), "model");
        assert!(component.init().is_ok());
        assert!(component.run().is_ok());
        assert!(component.pause().is_ok());
        assert!(component.stop().is_ok());
        assert_eq!(component.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn test_model_component_ports() {
        let config = ModelConfig {
            id: "port-test".into(),
            provider: "p".into(),
            model_name: "m".into(),
            base_url: "http://localhost".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Medium,
        };
        let component = ModelComponent::new(config);
        let ports = component.ports();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].direction, PortDirection::Input);
        assert_eq!(ports[1].direction, PortDirection::Output);
    }

    #[test]
    fn test_secure_component_permissions() {
        let config = ModelConfig {
            id: "perm-test".into(),
            provider: "p".into(),
            model_name: "m".into(),
            base_url: "http://localhost".into(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::High,
        };
        let component = ModelComponent::new(config);
        let perms = component.required_permissions();
        assert_eq!(perms.len(), 1);
        assert!(matches!(perms[0], Permission::NetworkAccess));
    }

    #[test]
    fn test_create_model_component() {
        let config = ModelConfig {
            id: "factory-test".into(),
            provider: "p".into(),
            model_name: "m".into(),
            base_url: "http://localhost".into(),
            api_key: "key".into(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Free,
        };
        let component = create_model_component(config);
        assert_eq!(component.config().id, "factory-test");
    }
}
