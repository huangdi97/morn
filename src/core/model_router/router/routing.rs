use super::qa_router;
use super::{
    ConfiguredModel, HybridStrategy, ModelRouter, ModelSpec, ModelType, ProviderCatalogEntry,
    RoutedModel, RouterMode,
};
use crate::core::error::MornError;

impl ModelRouter {
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

    pub fn mode(&self) -> &RouterMode {
        &self.mode
    }

    pub fn switch_mode(&mut self, mode: RouterMode) {
        self.mode = mode;
    }

    pub fn route(&self, prompt: &str) -> Result<RoutedModel, MornError> {
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

    pub fn select_model(
        &self,
        prompt: &str,
        capabilities: &[&str],
    ) -> Result<&ModelSpec, MornError> {
        match self.mode {
            RouterMode::CloudFirst => self.select_cloud(prompt, capabilities),
            RouterMode::LocalOnly => self.select_local(capabilities),
            RouterMode::Hybrid => self.select_hybrid(prompt, capabilities),
        }
    }

    fn select_cloud(&self, _prompt: &str, capabilities: &[&str]) -> Result<&ModelSpec, MornError> {
        let candidates: Vec<&ModelSpec> = self
            .cloud_models
            .iter()
            .filter(|m| m.is_available && qa_router::has_all_capabilities(m, capabilities))
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
            .ok_or_else(|| MornError::Internal("no available model".to_string()))
    }

    fn select_local(&self, capabilities: &[&str]) -> Result<&ModelSpec, MornError> {
        let candidates: Vec<&ModelSpec> = self
            .local_models
            .iter()
            .filter(|m| m.is_available && qa_router::has_all_capabilities(m, capabilities))
            .collect();
        if let Some(model) = candidates.first() {
            return Ok(model);
        }
        self.fallback_models
            .iter()
            .find(|m| m.is_available)
            .ok_or_else(|| MornError::Internal("no local model available".to_string()))
    }

    fn select_hybrid(&self, prompt: &str, capabilities: &[&str]) -> Result<&ModelSpec, MornError> {
        let local_available = self.has_available_local_model(capabilities);

        match self.hybrid_strategy {
            HybridStrategy::Auto => {
                if qa_router::estimate_complexity(prompt) > self.hybrid_threshold && local_available
                {
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

    fn route_from_spec(&self, spec: &ModelSpec) -> RoutedModel {
        let provider = self.find_provider(&spec.provider);
        let base_url = provider
            .map(|provider| provider.endpoint.clone())
            .unwrap_or_else(|| qa_router::endpoint_from_provider(&spec.provider));

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

    pub(super) fn find_provider(&self, provider: &str) -> Option<&ProviderCatalogEntry> {
        self.providers.iter().find(|p| p.name == provider)
    }

    fn has_available_local_model(&self, capabilities: &[&str]) -> bool {
        self.local_models
            .iter()
            .any(|m| m.is_available && qa_router::has_all_capabilities(m, capabilities))
    }

    fn estimate_cloud_cost(&self, prompt: &str, capabilities: &[&str]) -> f64 {
        let tokens = ((prompt.len() as f64) / 4.0).ceil().max(1.0);
        self.cloud_models
            .iter()
            .filter(|m| m.is_available && qa_router::has_all_capabilities(m, capabilities))
            .map(|m| (tokens / 1000.0) * m.cost_per_1k_tokens)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
}
