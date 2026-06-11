//! Router — model routing and provider selection.
use super::local_engine::LocalEngine;
use super::{
    ConfiguredModel, HybridStrategy, ModelRouter, ModelSpec, ModelType, ProviderCatalogEntry,
    RouterMode, RoutedModel,
};
use crate::config::ModelConfig as AppModelConfig;

impl ModelRouter {
    pub fn new() -> Self {
        let mut router = ModelRouter {
            mode: RouterMode::CloudFirst,
            local_engine: LocalEngine::new(),
            default_model: None,
            providers: default_provider_catalog(),
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

    pub fn with_default_model(
        provider: &str,
        name: &str,
        base_url: &str,
        api_key: Option<String>,
    ) -> Self {
        let mut router = Self::new();
        router.set_default_model(provider, name, base_url, api_key);
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

    pub fn set_default_model(
        &mut self,
        provider: &str,
        name: &str,
        base_url: &str,
        api_key: Option<String>,
    ) {
        let configured = ConfiguredModel {
            provider: provider.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key,
        };

        if !self
            .cloud_models
            .iter()
            .any(|model| model.provider == configured.provider && model.id == configured.name)
        {
            self.cloud_models.push(ModelSpec {
                id: configured.name.clone(),
                name: configured.name.clone(),
                provider: configured.provider.clone(),
                model_type: ModelType::Cloud,
                capabilities: vec!["chat".to_string(), "reasoning".to_string()],
                cost_per_1k_tokens: 0.0,
                is_available: true,
            });
        }

        self.default_model = Some(configured);
    }

    pub fn default_model(&self) -> Option<&ConfiguredModel> {
        self.default_model.as_ref()
    }

    fn init_default_models(&mut self) {
        for p in self.providers.clone() {
            #[cfg(feature = "providers-full")]
            {
                for model_id in &p.models {
                    let provider_name = p.name.as_str();
                    let is_local = is_local_provider(provider_name);
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
            {
                let _ = &p;
            }
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

    pub fn mode(&self) -> &RouterMode {
        &self.mode
    }

    pub fn switch_mode(&mut self, mode: RouterMode) {
        self.mode = mode;
    }

    pub fn route(&self, prompt: &str) -> Result<RoutedModel, String> {
        let selected = self.select_model(prompt, &["chat"])?;

        if selected.model_type == ModelType::Cloud {
            if let Some(default_model) = &self.default_model {
                return Ok(RoutedModel {
                    provider: default_model.provider.clone(),
                    name: default_model.name.clone(),
                    base_url: default_model.base_url.clone(),
                    api_key: default_model.api_key.clone(),
                    api_key_header: self
                        .get_provider_api_key_header(&default_model.provider)
                        .unwrap_or_else(|| "Authorization".to_string()),
                    model_type: ModelType::Cloud,
                });
            }
        }

        Ok(self.route_from_spec(selected))
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

    pub fn select_model(&self, prompt: &str, capabilities: &[&str]) -> Result<&ModelSpec, String> {
        match self.mode {
            RouterMode::CloudFirst => self.select_cloud(prompt, capabilities),
            RouterMode::LocalOnly => self.select_local(capabilities),
            RouterMode::Hybrid => self.select_hybrid(prompt, capabilities),
        }
    }

    fn select_cloud(&self, _prompt: &str, capabilities: &[&str]) -> Result<&ModelSpec, String> {
        let candidates: Vec<&ModelSpec> = self
            .cloud_models
            .iter()
            .filter(|m| m.is_available && has_all_capabilities(m, capabilities))
            .collect();
        if let Some(best) = candidates.iter().min_by(|a, b| {
            a.cost_per_1k_tokens
                .partial_cmp(&b.cost_per_1k_tokens)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return Ok(best);
        }
        self.fallback_models
            .iter()
            .find(|m| m.is_available)
            .ok_or_else(|| "no available model".to_string())
    }

    fn select_local(&self, capabilities: &[&str]) -> Result<&ModelSpec, String> {
        let candidates: Vec<&ModelSpec> = self
            .local_models
            .iter()
            .filter(|m| m.is_available && has_all_capabilities(m, capabilities))
            .collect();
        if let Some(model) = candidates.first() {
            return Ok(model);
        }
        self.fallback_models
            .iter()
            .find(|m| m.is_available)
            .ok_or_else(|| "no local model available".to_string())
    }

    fn select_hybrid(&self, prompt: &str, capabilities: &[&str]) -> Result<&ModelSpec, String> {
        let local_available = self.has_available_local_model(capabilities);

        match self.hybrid_strategy {
            HybridStrategy::Auto => {
                if estimate_complexity(prompt) > self.hybrid_threshold && local_available {
                    self.select_local(capabilities)
                } else {
                    self.select_cloud(prompt, capabilities)
                }
            }
            HybridStrategy::LocalFirst => {
                if local_available {
                    self.select_local(capabilities)
                } else {
                    self.select_cloud(prompt, capabilities)
                }
            }
            HybridStrategy::CloudOnly => self.select_cloud(prompt, capabilities),
            HybridStrategy::CostSave => {
                if self.estimate_cloud_cost(prompt, capabilities) > 0.01 && local_available {
                    self.select_local(capabilities)
                } else {
                    self.select_cloud(prompt, capabilities)
                }
            }
        }
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

    pub fn get_fallback_chain(&self) -> Vec<&ModelSpec> {
        let mut chain = Vec::new();
        match self.mode {
            RouterMode::CloudFirst => {
                chain.extend(self.cloud_models.iter().filter(|m| m.is_available));
                if cfg!(feature = "local-llm") {
                    chain.extend(self.local_models.iter().filter(|m| m.is_available));
                }
                chain.extend(self.fallback_models.iter().filter(|m| m.is_available));
            }
            RouterMode::LocalOnly => {
                if cfg!(feature = "local-llm") {
                    chain.extend(self.local_models.iter().filter(|m| m.is_available));
                }
                chain.extend(self.cloud_models.iter().filter(|m| m.is_available));
                chain.extend(self.fallback_models.iter().filter(|m| m.is_available));
            }
            RouterMode::Hybrid => {
                if cfg!(feature = "local-llm") {
                    chain.extend(self.local_models.iter().filter(|m| m.is_available));
                }
                chain.extend(self.cloud_models.iter().filter(|m| m.is_available));
                chain.extend(self.fallback_models.iter().filter(|m| m.is_available));
            }
        }
        chain
    }

    pub fn fallback_routes_for(&self, current: &RoutedModel) -> Vec<RoutedModel> {
        self.get_fallback_chain()
            .into_iter()
            .map(|spec| self.route_from_spec(spec))
            .filter(|route| {
                route.provider != current.provider
                    || route.name != current.name
                    || route.base_url != current.base_url
            })
            .collect()
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

    fn route_from_spec(&self, spec: &ModelSpec) -> RoutedModel {
        let provider = self.find_provider(&spec.provider);
        let base_url = provider
            .map(|provider| provider.endpoint.clone())
            .unwrap_or_else(|| endpoint_from_provider(&spec.provider));

        RoutedModel {
            provider: spec.provider.clone(),
            name: spec.id.clone(),
            base_url,
            api_key: provider.and_then(|provider| provider.api_key.clone()),
            api_key_header: provider
                .map(|provider| provider.api_key_header.clone())
                .unwrap_or_else(|| "Authorization".to_string()),
            model_type: spec.model_type,
        }
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

    fn find_provider(&self, provider: &str) -> Option<&ProviderCatalogEntry> {
        self.providers.iter().find(|p| p.name == provider)
    }

    fn has_available_local_model(&self, capabilities: &[&str]) -> bool {
        self.local_models
            .iter()
            .any(|m| m.is_available && has_all_capabilities(m, capabilities))
    }

    fn estimate_cloud_cost(&self, prompt: &str, capabilities: &[&str]) -> f64 {
        let tokens = ((prompt.len() as f64) / 4.0).ceil().max(1.0);
        self.cloud_models
            .iter()
            .filter(|m| m.is_available && has_all_capabilities(m, capabilities))
            .map(|m| (tokens / 1000.0) * m.cost_per_1k_tokens)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

fn has_all_capabilities(model: &ModelSpec, required: &[&str]) -> bool {
    required
        .iter()
        .all(|cap| model.capabilities.iter().any(|c| c == cap))
}

fn default_provider_catalog() -> Vec<ProviderCatalogEntry> {
    super::providers::PROVIDERS
        .iter()
        .map(|provider| ProviderCatalogEntry {
            name: provider.name.to_string(),
            endpoint: provider.endpoint.to_string(),
            api_key_header: provider.api_key_header.to_string(),
            models: provider
                .models
                .iter()
                .map(|model| model.to_string())
                .collect(),
            api_key: None,
        })
        .collect()
}

fn is_local_provider(provider: &str) -> bool {
    provider == "ollama" || provider == "lm_studio" || provider == "local"
}

fn estimate_complexity(prompt: &str) -> usize {
    let lower = prompt.to_ascii_lowercase();
    let keyword_score = [
        "analyze", "compare", "explain", "write", "code", "generate", "design", "plan",
    ]
    .iter()
    .filter(|keyword| lower.contains(**keyword))
    .count()
        * 100;

    prompt.len() + keyword_score
}

fn endpoint_from_provider(provider: &str) -> String {
    if provider.starts_with("http://") || provider.starts_with("https://") {
        provider.to_string()
    } else if provider.contains('.') {
        format!("https://{}", provider)
    } else {
        String::new()
    }
}