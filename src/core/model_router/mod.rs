//! model_router — Routes LLM requests to appropriate provider based on mode and capabilities.
pub mod providers;
pub mod local_engine;

#[derive(Debug, Clone, PartialEq)]
pub enum RouterMode {
    CloudFirst,
    LocalOnly,
    Hybrid,
}

impl RouterMode {
    pub fn route(&self, request: &str) -> Result<String, String> {
        match self {
            RouterMode::CloudFirst => Ok("cloud".to_string()),
            RouterMode::LocalOnly => Ok("local".to_string()),
            RouterMode::Hybrid => {
                if request.len() < 100 { Ok("local".to_string()) }
                else { Ok("cloud".to_string()) }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_type: ModelType,
    pub capabilities: Vec<String>,
    pub cost_per_1k_tokens: f64,
    pub is_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelType {
    Cloud,
    LocalGGUF,
    FallbackTiny,
}

#[derive(Debug, Clone)]
pub struct ModelRouter {
    mode: RouterMode,
    cloud_models: Vec<ModelSpec>,
    local_models: Vec<ModelSpec>,
    fallback_models: Vec<ModelSpec>,
    hybrid_threshold: usize,
    gguf_discovered: bool,
}

impl ModelRouter {
    pub fn new() -> Self {
        let mut router = ModelRouter {
            mode: RouterMode::CloudFirst,
            cloud_models: Vec::new(),
            local_models: Vec::new(),
            fallback_models: Vec::new(),
            hybrid_threshold: 500,
            gguf_discovered: false,
        };
        router.init_default_models();
        router
    }

    fn init_default_models(&mut self) {
        for p in providers::PROVIDERS {
            #[cfg(feature = "providers-full")]
            {
                for model_id in p.models {
                    let provider_name = p.name;
                    let is_local = provider_name == "ollama"
                        || provider_name == "lm_studio"
                        || provider_name == "local";
                    let model_type = if is_local {
                        ModelType::LocalGGUF
                    } else if provider_name == "builtin" {
                        ModelType::FallbackTiny
                    } else {
                        ModelType::Cloud
                    };
                    let spec = ModelSpec {
                        id: model_id.to_string(),
                        name: model_id.to_string(),
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
                let _ = p;
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
                capabilities: vec!["chat".to_string(), "vision".to_string(), "reasoning".to_string()],
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

    fn select_cloud(
        &self,
        _prompt: &str,
        capabilities: &[&str],
    ) -> Result<&ModelSpec, String> {
        let candidates: Vec<&ModelSpec> = self
            .cloud_models
            .iter()
            .filter(|m| m.is_available && has_all_capabilities(m, capabilities))
            .collect();
        if let Some(best) = candidates
            .iter()
            .min_by(|a, b| a.cost_per_1k_tokens.partial_cmp(&b.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal))
        {
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

    fn select_hybrid(
        &self,
        prompt: &str,
        capabilities: &[&str],
    ) -> Result<&ModelSpec, String> {
        let is_complex = prompt.len() > self.hybrid_threshold
            || prompt.contains("analyze")
            || prompt.contains("compare")
            || prompt.contains("explain")
            || prompt.contains("write")
            || prompt.contains("code")
            || prompt.contains("generate");

        if is_complex {
            self.select_cloud(prompt, capabilities)
        } else if cfg!(feature = "local-llm") && !self.local_models.is_empty() {
            self.select_local(capabilities)
        } else {
            self.select_cloud(prompt, capabilities)
        }
    }

    pub fn discover_local_models(&mut self, gguf_dir: &str) -> Vec<String> {
        let mut found = Vec::new();
        if cfg!(feature = "local-llm") {
            if let Ok(entries) = std::fs::read_dir(gguf_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
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

    pub fn set_hybrid_threshold(&mut self, tokens: usize) {
        self.hybrid_threshold = tokens;
    }

    pub fn get_provider_endpoint(&self, provider: &str) -> Option<&'static str> {
        providers::get_provider(provider).map(|p| p.endpoint)
    }

    pub fn get_provider_api_key_header(&self, provider: &str) -> Option<&'static str> {
        providers::get_provider(provider).map(|p| p.api_key_header)
    }

    pub fn get_provider_models(&self, provider: &str) -> Option<&'static [&'static str]> {
        providers::get_provider(provider).map(|p| p.models)
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;