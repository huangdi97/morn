use super::local_engine::LocalEngine;
use super::{
    ConfiguredModel, HybridStrategy, ModelRouter, ModelSpec, ModelType, ProviderCatalogEntry,
    RoutedModel, RouterMode,
};
use crate::config::ModelConfig as AppModelConfig;

pub mod qa_router;
pub mod routing;

impl ModelRouter {
    pub fn new() -> Self {
        let mut router = ModelRouter {
            mode: RouterMode::CloudFirst,
            local_engine: LocalEngine::new(),
            default_model: None,
            providers: qa_router::default_provider_catalog(),
            cloud_models: Vec::new(),
            local_models: Vec::new(),
            fallback_models: Vec::new(),
            hybrid_strategy: HybridStrategy::Auto,
            hybrid_threshold: 500,
            gguf_discovered: false,
        };
        router.init_default_models();
        router
    }

    pub fn from_model_config(config: &AppModelConfig) -> Self {
        let mut router = Self::new();
        router.apply_model_config(config);
        router
    }

    pub fn apply_model_config(&mut self, config: &AppModelConfig) {
        self.merge_custom_providers(config);
        self.hybrid_threshold = config.hybrid.complexity_threshold;
        self.hybrid_strategy = HybridStrategy::parse(&config.hybrid.strategy).unwrap_or_default();
        if config.local_first && self.hybrid_strategy != HybridStrategy::CloudOnly {
            self.hybrid_strategy = HybridStrategy::LocalFirst;
        }

        let api_key = config.api_key.clone().or_else(|| {
            self.find_provider(&config.provider)
                .and_then(|provider| provider.api_key.clone())
        });
        let base_url = config
            .providers
            .get(&config.provider)
            .map(|provider| provider.base_url.clone())
            .filter(|base_url| !base_url.trim().is_empty())
            .unwrap_or_else(|| config.base_url.clone());

        self.set_default_model(&config.provider, &config.name, &base_url, api_key);
    }

    fn init_default_models(&mut self) {
        for p in self.providers.clone() {
            #[cfg(feature = "providers-full")]
            {
                for model_id in &p.models {
                    let provider_name = p.name.as_str();
                    let is_local = qa_router::is_local_provider(provider_name);
                    let model_type = if is_local {
                        ModelType::LocalGGUF
                    } else if provider_name == "builtin" {
                        ModelType::FallbackTiny
                    } else {
                        ModelType::Cloud
                    };
                    let spec = ModelSpec {
                        id: model_id.clone(),
                        name: model_id.clone(),
                        provider: provider_name.to_string(),
                        model_type,
                        capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                        cost_per_1k_tokens: 0.01,
                        is_available: true,
                    };
                    match model_type {
                        ModelType::Cloud => self.cloud_models.push(spec),
                        ModelType::LocalGGUF => self.local_models.push(spec),
                        ModelType::FallbackTiny => self.fallback_models.push(spec),
                    }
                }
            }
            #[cfg(not(feature = "providers-full"))]
            {}
        }

        #[cfg(not(feature = "providers-full"))]
        {
            self.cloud_models.push(ModelSpec {
                id: "deepseek-chat".to_string(),
                name: "DeepSeek Chat".to_string(),
                provider: "api.deepseek.com".to_string(),
                model_type: ModelType::Cloud,
                capabilities: vec![
                    "chat".to_string(),
                    "reasoning".to_string(),
                    "code".to_string(),
                ],
                cost_per_1k_tokens: 0.014,
                is_available: true,
            });
            self.cloud_models.push(ModelSpec {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: "openai".to_string(),
                model_type: ModelType::Cloud,
                capabilities: vec![
                    "chat".to_string(),
                    "vision".to_string(),
                    "reasoning".to_string(),
                ],
                cost_per_1k_tokens: 0.01,
                is_available: true,
            });
            self.cloud_models.push(ModelSpec {
                id: "claude-3-sonnet".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                model_type: ModelType::Cloud,
                capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                cost_per_1k_tokens: 0.015,
                is_available: true,
            });
            self.cloud_models.push(ModelSpec {
                id: "sensenova".to_string(),
                name: "SenseNova".to_string(),
                provider: "sensenova".to_string(),
                model_type: ModelType::Cloud,
                capabilities: vec!["chat".to_string()],
                cost_per_1k_tokens: 0.008,
                is_available: true,
            });
            self.fallback_models.push(ModelSpec {
                id: "tiny-llm".to_string(),
                name: "Tiny Fallback LLM".to_string(),
                provider: "builtin".to_string(),
                model_type: ModelType::FallbackTiny,
                capabilities: vec!["chat".to_string()],
                cost_per_1k_tokens: 0.001,
                is_available: true,
            });
        }
    }

    pub fn available_models(&self) -> Vec<&ModelSpec> {
        let mut models: Vec<&ModelSpec> = self
            .cloud_models
            .iter()
            .filter(|m| m.is_available)
            .collect();
        if cfg!(feature = "local-llm") {
            models.extend(self.local_models.iter().filter(|m| m.is_available));
        }
        models.extend(self.fallback_models.iter().filter(|m| m.is_available));
        models
    }

    pub fn discover_local_models(&mut self, gguf_dir: &str) -> Vec<String> {
        let mut found = Vec::new();
        if cfg!(feature = "local-llm") {
            if let Ok(models) = LocalEngine::discover(gguf_dir) {
                self.local_engine.models = models;
                for model in &self.local_engine.models {
                    let name = model.name.clone();
                    let id = format!("local_{}", name);
                    self.local_models.push(ModelSpec {
                        id: id.clone(),
                        name: name.clone(),
                        provider: "local".to_string(),
                        model_type: ModelType::LocalGGUF,
                        capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                        cost_per_1k_tokens: 0.0,
                        is_available: true,
                    });
                    found.push(id);
                }
            }
            if !found.is_empty() {
                self.gguf_discovered = true;
            }
        }
        found
    }

    pub fn has_local_models(&self) -> bool {
        if cfg!(feature = "local-llm") {
            !self.local_models.is_empty()
        } else {
            false
        }
    }

    pub fn gguf_discovered(&self) -> bool {
        self.gguf_discovered
    }

    pub fn set_hybrid_threshold(&mut self, tokens: usize) {
        self.hybrid_threshold = tokens;
    }

    pub fn set_hybrid_strategy(&mut self, strategy: HybridStrategy) {
        self.hybrid_strategy = strategy;
    }

    pub fn hybrid_strategy(&self) -> HybridStrategy {
        self.hybrid_strategy
    }

    pub fn get_provider_endpoint(&self, provider: &str) -> Option<String> {
        self.find_provider(provider)
            .map(|provider| provider.endpoint.clone())
    }

    pub fn get_provider_api_key_header(&self, provider: &str) -> Option<String> {
        self.find_provider(provider)
            .map(|provider| provider.api_key_header.clone())
    }

    pub fn get_provider_models(&self, provider: &str) -> Option<Vec<String>> {
        self.find_provider(provider)
            .map(|provider| provider.models.clone())
    }

    fn merge_custom_providers(&mut self, config: &AppModelConfig) {
        for (name, provider) in &config.providers {
            let entry = ProviderCatalogEntry {
                name: name.clone(),
                endpoint: provider.base_url.clone(),
                api_key_header: provider.api_key_header.clone(),
                models: provider.models.clone(),
                api_key: provider.api_key.clone(),
            };

            if let Some(existing) = self
                .providers
                .iter_mut()
                .find(|existing| existing.name == entry.name)
            {
                *existing = entry.clone();
            } else {
                self.providers.push(entry.clone());
            }

            for model_id in &entry.models {
                if self
                    .cloud_models
                    .iter()
                    .any(|model| model.provider == entry.name && model.id == *model_id)
                {
                    continue;
                }

                self.cloud_models.push(ModelSpec {
                    id: model_id.clone(),
                    name: model_id.clone(),
                    provider: entry.name.clone(),
                    model_type: ModelType::Cloud,
                    capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                    cost_per_1k_tokens: 0.01,
                    is_available: true,
                });
            }
        }
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}
