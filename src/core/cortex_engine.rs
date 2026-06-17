//! cortex_engine — Coordinates high-level cognitive processing across agents and tasks.
use crate::core::error::MornError;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelProfile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub version: String,
    pub context_length: usize,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPMarketPlugin {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub source_url: String,
    pub installed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CortexConfig {
    pub default_model: String,
    pub temperature: f64,
    pub max_tokens: usize,
    pub top_p: f64,
}

impl Default for CortexConfig {
    fn default() -> Self {
        CortexConfig {
            default_model: "deepseek-chat".into(),
            temperature: 0.7,
            max_tokens: 4096,
            top_p: 0.9,
        }
    }
}

pub struct CortexEngine {
    config: CortexConfig,
    models: HashMap<String, ModelProfile>,
    mcp_plugins: Mutex<Vec<MCPMarketPlugin>>,
    mcp_registry_url: String,
}

impl CortexEngine {
    pub fn new(config: CortexConfig) -> Self {
        let mut models = HashMap::new();
        models.insert(
            "deepseek-chat".into(),
            ModelProfile {
                id: "deepseek-chat".into(),
                name: "DeepSeek Chat".into(),
                provider: "deepseek".into(),
                version: "2.0".into(),
                context_length: 65536,
                capabilities: vec!["chat".into(), "reasoning".into(), "code".into()],
            },
        );
        models.insert(
            "deepseek-reasoner".into(),
            ModelProfile {
                id: "deepseek-reasoner".into(),
                name: "DeepSeek Reasoner".into(),
                provider: "deepseek".into(),
                version: "1.0".into(),
                context_length: 65536,
                capabilities: vec!["reasoning".into(), "analysis".into()],
            },
        );

        CortexEngine {
            config,
            models,
            mcp_plugins: Mutex::new(Vec::new()),
            mcp_registry_url: "https://registry.morn.ai/mcp/plugins".into(),
        }
    }

    pub fn config(&self) -> &CortexConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut CortexConfig {
        &mut self.config
    }

    pub fn list_models(&self) -> Vec<&ModelProfile> {
        self.models.values().collect()
    }

    pub fn get_model(&self, id: &str) -> Option<&ModelProfile> {
        self.models.get(id)
    }

    pub fn register_model(&mut self, profile: ModelProfile) {
        self.models.insert(profile.id.clone(), profile);
    }

    pub fn mcp_plugins(&self) -> Vec<MCPMarketPlugin> {
        self.mcp_plugins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn mcp_market(
        &self,
        registry_url: Option<&str>,
    ) -> Result<Vec<MCPMarketPlugin>, MornError> {
        let url = registry_url.unwrap_or(&self.mcp_registry_url);

        let response = reqwest::blocking::get(url).map_err(|e| {
            MornError::Internal(format!("Failed to fetch MCP market from '{}': {}", url, e))
        })?;

        if !response.status().is_success() {
            return Err(MornError::Internal(format!(
                "MCP market registry returned HTTP {} from '{}'",
                response.status(),
                url
            )));
        }

        let plugins: Vec<MCPMarketPlugin> = response.json().map_err(|e| {
            MornError::Internal(format!("Failed to parse MCP market response: {}", e))
        })?;

        let mut installed = self
            .mcp_plugins
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        *installed = plugins.clone();

        Ok(plugins)
    }

    pub fn set_mcp_registry_url(&mut self, url: String) {
        self.mcp_registry_url = url;
    }

    pub fn mcp_registry_url(&self) -> &str {
        &self.mcp_registry_url
    }
}

impl Default for CortexEngine {
    fn default() -> Self {
        CortexEngine::new(CortexConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortex_engine_default() {
        let engine = CortexEngine::default();
        assert_eq!(engine.config().default_model, "deepseek-chat");
        assert_eq!(engine.list_models().len(), 2);
    }

    #[test]
    fn test_list_models() {
        let engine = CortexEngine::default();
        let models = engine.list_models();
        assert!(models.iter().any(|m| m.id == "deepseek-chat"));
        assert!(models.iter().any(|m| m.id == "deepseek-reasoner"));
    }

    #[test]
    fn test_get_model() {
        let engine = CortexEngine::default();
        let model = engine.get_model("deepseek-chat");
        assert!(model.is_some());
        assert_eq!(model.unwrap().provider, "deepseek");
    }

    #[test]
    fn test_get_model_not_found() {
        let engine = CortexEngine::default();
        assert!(engine.get_model("nonexistent").is_none());
    }

    #[test]
    fn test_register_model() {
        let mut engine = CortexEngine::default();
        let profile = ModelProfile {
            id: "custom-model".into(),
            name: "Custom Model".into(),
            provider: "custom".into(),
            version: "1.0".into(),
            context_length: 8192,
            capabilities: vec!["chat".into()],
        };
        engine.register_model(profile);
        assert_eq!(engine.list_models().len(), 3);
        assert!(engine.get_model("custom-model").is_some());
    }

    #[test]
    fn test_config_mut() {
        let mut engine = CortexEngine::default();
        engine.config_mut().temperature = 0.5;
        assert_eq!(engine.config().temperature, 0.5);
    }

    #[test]
    fn test_mcp_plugins_empty() {
        let engine = CortexEngine::default();
        assert!(engine.mcp_plugins().is_empty());
    }

    #[test]
    fn test_set_mcp_registry_url() {
        let mut engine = CortexEngine::default();
        engine.set_mcp_registry_url("https://custom.registry.com/plugins".into());
        assert_eq!(
            engine.mcp_registry_url(),
            "https://custom.registry.com/plugins"
        );
    }

    #[test]
    fn test_mcp_market_bad_url() {
        let engine = CortexEngine::default();
        let result = engine.mcp_market(Some("https://0.0.0.0/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_cortex_config_default() {
        let config = CortexConfig::default();
        assert_eq!(config.default_model, "deepseek-chat");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.top_p, 0.9);
    }

    #[test]
    fn test_model_profile_serialization() {
        let profile = ModelProfile {
            id: "test".into(),
            name: "Test".into(),
            provider: "test".into(),
            version: "1.0".into(),
            context_length: 4096,
            capabilities: vec!["chat".into()],
        };
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: ModelProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
        assert_eq!(deserialized.context_length, 4096);
    }
}
