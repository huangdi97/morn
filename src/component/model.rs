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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(id: &str, provider: &str, model: &str) -> ModelConfig {
        ModelConfig {
            id: id.into(),
            provider: provider.into(),
            model_name: model.into(),
            base_url: format!("https://api.{}.com", provider),
            api_key: "test-key".into(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Low,
        }
    }

    #[test]
    fn test_model_parameters_default() {
        let p = ModelParameters::default();
        assert_eq!(p.temperature, 0.6);
        assert_eq!(p.max_tokens, 4096);
        assert_eq!(p.top_p, 0.9);
    }

    #[test]
    fn test_model_config_new() {
        let config = make_config("test-model", "deepseek", "deepseek-chat");
        assert_eq!(config.id, "test-model");
        assert_eq!(config.provider, "deepseek");
        assert_eq!(config.model_name, "deepseek-chat");
        assert_eq!(config.base_url, "https://api.deepseek.com");
    }

    #[test]
    fn test_model_config_with_different_providers() {
        let openai = make_config("openai-gpt4", "openai", "gpt-4");
        assert_eq!(openai.provider, "openai");
        assert_eq!(openai.model_name, "gpt-4");

        let anthropic = make_config("anthropic-claude", "anthropic", "claude-3");
        assert_eq!(anthropic.provider, "anthropic");
        assert_eq!(anthropic.model_name, "claude-3");
    }

    #[test]
    fn test_model_config_with_fallback() {
        let mut config = make_config("primary", "deepseek", "deepseek-chat");
        config.fallback = Some("backup".into());
        assert_eq!(config.fallback, Some("backup".into()));
    }

    #[test]
    fn test_model_config_cost_tier() {
        let mut config = make_config("test", "provider", "model");
        assert!(matches!(config.cost_tier, CostTier::Low));

        config.cost_tier = CostTier::High;
        assert!(matches!(config.cost_tier, CostTier::High));
    }

    #[test]
    fn test_model_config_custom_parameters() {
        let config = ModelConfig {
            id: "custom".into(),
            provider: "provider".into(),
            model_name: "model".into(),
            base_url: "https://custom.api.com".into(),
            api_key: "key".into(),
            parameters: ModelParameters {
                temperature: 0.1,
                max_tokens: 1024,
                top_p: 0.5,
            },
            fallback: None,
            cost_tier: CostTier::Free,
        };
        assert_eq!(config.parameters.temperature, 0.1);
        assert_eq!(config.parameters.max_tokens, 1024);
        assert_eq!(config.parameters.top_p, 0.5);
    }

    #[test]
    fn test_model_component_new() {
        let config = make_config("mc1", "deepseek", "deepseek-chat");
        let comp = ModelComponent::new(config);
        assert_eq!(comp.id(), "mc1");
        assert_eq!(comp.type_name(), "model");
        assert_eq!(comp.config().model_name, "deepseek-chat");
    }

    #[test]
    fn test_model_component_chat() {
        let config = make_config("chat-test", "deepseek", "deepseek-chat");
        let comp = ModelComponent::new(config);
        let response = comp.chat("hello", "system prompt").unwrap();
        assert_eq!(response, "[deepseek:deepseek-chat] (simulated response)");
    }

    #[test]
    fn test_model_component_chat_different_provider() {
        let config = make_config("chat-test-2", "openai", "gpt-4");
        let comp = ModelComponent::new(config);
        let response = comp.chat("hello", "").unwrap();
        assert_eq!(response, "[openai:gpt-4] (simulated response)");
    }

    #[test]
    fn test_model_component_lifecycle() {
        let config = make_config("lifecycle", "test", "model");
        let mut comp = ModelComponent::new(config);
        assert!(comp.init().is_ok());
        assert!(comp.run().is_ok());
        assert!(comp.pause().is_ok());
        assert!(comp.stop().is_ok());
        assert_eq!(comp.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn test_model_component_ports() {
        let config = make_config("ports-test", "test", "model");
        let comp = ModelComponent::new(config);
        let ports = comp.ports();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].id, "input");
        assert_eq!(
            ports[0].direction.clone() as i32,
            PortDirection::Input as i32
        );
        assert_eq!(ports[1].id, "output");
        assert_eq!(
            ports[1].direction.clone() as i32,
            PortDirection::Output as i32
        );
    }

    #[test]
    fn test_model_component_requires_network() {
        let config = make_config("network-test", "test", "model");
        let comp = ModelComponent::new(config);
        let perms = comp.required_permissions();
        assert_eq!(perms.len(), 1);
        assert!(matches!(perms[0], Permission::NetworkAccess));
    }

    #[test]
    fn test_create_default_models() {
        let models = create_default_models();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "model-deepseek-chat");
        assert_eq!(models[1].id, "model-deepseek-reasoner");
    }

    #[test]
    fn test_create_model_component() {
        let config = make_config("factory-test", "deepseek", "deepseek-chat");
        let comp = create_model_component(config);
        assert_eq!(comp.id(), "factory-test");
    }

    #[test]
    fn test_cost_tier_variants() {
        let tiers = vec![
            CostTier::Free,
            CostTier::Low,
            CostTier::Medium,
            CostTier::High,
        ];
        assert_eq!(tiers.len(), 4);
    }
}
