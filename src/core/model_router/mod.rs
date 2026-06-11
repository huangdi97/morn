//! model_router — Routes LLM requests to appropriate provider based on mode and capabilities.
pub mod local_engine;
pub mod providers;
pub mod router;
#[allow(unused_imports)]
pub use router::*;

use local_engine::LocalEngine;

#[derive(Debug, Clone, Copy, PartialEq)]
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
                if request.len() < 100 {
                    Ok("local".to_string())
                } else {
                    Ok("cloud".to_string())
                }
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

#[derive(Debug, Clone, PartialEq)]
pub struct ConfiguredModel {
    pub provider: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedModel {
    pub provider: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_key_header: String,
    pub model_type: ModelType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelType {
    Cloud,
    LocalGGUF,
    FallbackTiny,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HybridStrategy {
    #[default]
    Auto,
    LocalFirst,
    CloudOnly,
    CostSave,
}

impl HybridStrategy {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "auto" => Ok(HybridStrategy::Auto),
            "local_first" | "localfirst" => Ok(HybridStrategy::LocalFirst),
            "cloud_only" | "cloudonly" => Ok(HybridStrategy::CloudOnly),
            "cost_save" | "costsave" => Ok(HybridStrategy::CostSave),
            other => Err(format!("unknown hybrid strategy: {}", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProviderCatalogEntry {
    name: String,
    endpoint: String,
    api_key_header: String,
    models: Vec<String>,
    api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelRouter {
    mode: RouterMode,
    local_engine: LocalEngine,
    default_model: Option<ConfiguredModel>,
    providers: Vec<ProviderCatalogEntry>,
    cloud_models: Vec<ModelSpec>,
    local_models: Vec<ModelSpec>,
    fallback_models: Vec<ModelSpec>,
    hybrid_strategy: HybridStrategy,
    hybrid_threshold: usize,
    gguf_discovered: bool,
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
