use serde::Deserialize;
use std::collections::HashMap;
use std::env;

use super::{env_u64, env_usize};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub providers: HashMap<String, CustomProviderConfig>,
    pub hybrid: HybridConfig,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "deepseek".to_string(),
            name: "deepseek-chat".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            api_key: None,
            timeout_seconds: 300,
            providers: HashMap::new(),
            hybrid: HybridConfig::default(),
        }
    }
}

impl ModelConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            provider: env::var("MORN_MODEL_PROVIDER").unwrap_or(default.provider),
            name: env::var("MORN_MODEL_NAME").unwrap_or(default.name),
            base_url: env::var("MORN_MODEL_BASE_URL").unwrap_or(default.base_url),
            api_key: env::var("MORN_API_KEY").ok(),
            timeout_seconds: env_u64("MORN_MODEL_TIMEOUT_SECONDS", default.timeout_seconds),
            providers: HashMap::new(),
            hybrid: HybridConfig::from_env(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CustomProviderConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub models: Vec<String>,
    pub api_key_header: String,
}

impl Default for CustomProviderConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_key: None,
            models: Vec::new(),
            api_key_header: "Authorization".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HybridConfig {
    pub strategy: String,
    pub complexity_threshold: usize,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            strategy: "auto".to_string(),
            complexity_threshold: 500,
        }
    }
}

impl HybridConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            strategy: env::var("MORN_MODEL_HYBRID_STRATEGY").unwrap_or(default.strategy),
            complexity_threshold: env_usize(
                "MORN_MODEL_HYBRID_COMPLEXITY_THRESHOLD",
                default.complexity_threshold,
            ),
        }
    }
}
