//! Morn 配置系统 — TOML 配置加载、合并、校验、环境变量注入
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};

mod channels;
mod model;
mod server;

pub use channels::{
    ChannelsConfig, DaemonConfig, DingTalkConfig, MiniProgramConfig, PushPlusConfig,
    ServerChanConfig, TelegramConfig, WeComConfig,
};
pub use model::{CustomProviderConfig, HybridConfig, ModelConfig};
pub use server::ServerConfig;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct MornConfig {
    pub server: ServerConfig,
    pub model: ModelConfig,
    pub channels: ChannelsConfig,
    pub daemon: DaemonConfig,
}

impl MornConfig {
    pub fn load() -> Result<Self, String> {
        for path in Self::candidate_paths() {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        Ok(Self::from_env())
    }

    pub fn from_env() -> Self {
        Self {
            server: ServerConfig::from_env(),
            model: ModelConfig::from_env(),
            channels: ChannelsConfig::from_env(),
            daemon: DaemonConfig::from_env(),
        }
    }

    fn candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Ok(path) = env::var("MORN_CONFIG") {
            paths.push(expand_tilde(path));
        }

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("morn").join("config.toml"));
        }

        paths.push(PathBuf::from("morn.toml"));
        paths
    }

    fn from_file(path: &Path) -> Result<Self, String> {
        let config = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config {}: {}", path.display(), e))?;

        toml::from_str::<Self>(&config)
            .map_err(|e| format!("Failed to parse config {}: {}", path.display(), e))
    }
}

pub(crate) fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|value| match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

pub(crate) fn env_u16(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default)
}

pub(crate) fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

pub(crate) fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

pub(crate) fn env_path(key: &str, default: PathBuf) -> PathBuf {
    env::var(key).map(expand_tilde).unwrap_or(default)
}

pub(crate) fn expand_tilde(path: impl AsRef<str>) -> PathBuf {
    let path = path.as_ref();

    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(path)
}

pub(crate) fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = String::deserialize(deserializer)?;
    Ok(expand_tilde(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morn_config_default() {
        let config = MornConfig::default();
        assert!(!config.server.enabled);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3030);
        assert_eq!(config.model.provider, "deepseek");
        assert_eq!(config.model.name, "deepseek-chat");
        assert_eq!(config.model.timeout_seconds, 300);
        assert!(config.model.api_key.is_none());
        assert!(config.channels.cli);
        assert!(config.channels.desktop);
        assert!(!config.channels.rest_api);
        assert_eq!(config.channels.default_channel, "cli");
        assert!(!config.daemon.enabled);
    }

    #[test]
    fn test_server_config_default() {
        let srv = ServerConfig::default();
        assert!(!srv.enabled);
        assert_eq!(srv.host, "127.0.0.1");
        assert_eq!(srv.port, 3030);
        assert_eq!(srv.base_url, "http://127.0.0.1:3030");
    }

    #[test]
    fn test_model_config_default() {
        let model = ModelConfig::default();
        assert_eq!(model.provider, "deepseek");
        assert_eq!(model.name, "deepseek-chat");
        assert_eq!(model.base_url, "https://api.deepseek.com");
        assert!(model.api_key.is_none());
        assert_eq!(model.timeout_seconds, 300);
        assert!(model.providers.is_empty());
        assert_eq!(model.hybrid.strategy, "auto");
        assert_eq!(model.hybrid.complexity_threshold, 500);
    }

    #[test]
    fn test_model_config_from_env() {
        let original_provider = env::var("MORN_MODEL_PROVIDER").ok();
        let original_name = env::var("MORN_MODEL_NAME").ok();
        let original_base_url = env::var("MORN_MODEL_BASE_URL").ok();
        let original_key = env::var("MORN_API_KEY").ok();
        let original_timeout = env::var("MORN_MODEL_TIMEOUT_SECONDS").ok();

        env::set_var("MORN_MODEL_PROVIDER", "openai");
        env::set_var("MORN_MODEL_NAME", "gpt-4");
        env::set_var("MORN_MODEL_BASE_URL", "https://api.openai.com");
        env::set_var("MORN_API_KEY", "sk-test");
        env::set_var("MORN_MODEL_TIMEOUT_SECONDS", "120");

        let model = ModelConfig::from_env();
        assert_eq!(model.provider, "openai");
        assert_eq!(model.name, "gpt-4");
        assert_eq!(model.base_url, "https://api.openai.com");
        assert_eq!(model.api_key, Some("sk-test".to_string()));
        assert_eq!(model.timeout_seconds, 120);

        if let Some(v) = original_provider {
            env::set_var("MORN_MODEL_PROVIDER", v);
        } else {
            env::remove_var("MORN_MODEL_PROVIDER");
        }
        if let Some(v) = original_name {
            env::set_var("MORN_MODEL_NAME", v);
        } else {
            env::remove_var("MORN_MODEL_NAME");
        }
        if let Some(v) = original_base_url {
            env::set_var("MORN_MODEL_BASE_URL", v);
        } else {
            env::remove_var("MORN_MODEL_BASE_URL");
        }
        if let Some(v) = original_key {
            env::set_var("MORN_API_KEY", v);
        } else {
            env::remove_var("MORN_API_KEY");
        }
        if let Some(v) = original_timeout {
            env::set_var("MORN_MODEL_TIMEOUT_SECONDS", v);
        } else {
            env::remove_var("MORN_MODEL_TIMEOUT_SECONDS");
        }
    }

    #[test]
    fn test_model_config_from_env_fallback() {
        let original_provider = env::var("MORN_MODEL_PROVIDER").ok();
        let original_name = env::var("MORN_MODEL_NAME").ok();
        env::remove_var("MORN_MODEL_PROVIDER");
        env::remove_var("MORN_MODEL_NAME");

        let model = ModelConfig::from_env();
        assert_eq!(model.provider, "deepseek");
        assert_eq!(model.name, "deepseek-chat");

        if let Some(v) = original_provider {
            env::set_var("MORN_MODEL_PROVIDER", v);
        }
        if let Some(v) = original_name {
            env::set_var("MORN_MODEL_NAME", v);
        }
    }

    #[test]
    fn test_channels_config_default() {
        let ch = ChannelsConfig::default();
        assert!(ch.cli);
        assert!(ch.desktop);
        assert!(!ch.rest_api);
        assert!(!ch.telegram.enabled);
        assert_eq!(ch.default_channel, "cli");
    }

    #[test]
    fn test_daemon_config_default() {
        let d = DaemonConfig::default();
        assert!(!d.enabled);
        assert!(d.data_dir.to_string_lossy().contains("morn"));
    }

    #[test]
    fn test_hybrid_config_default() {
        let h = HybridConfig::default();
        assert_eq!(h.strategy, "auto");
        assert_eq!(h.complexity_threshold, 500);
    }

    #[test]
    fn test_channels_config_env_override() {
        let original_cli = env::var("MORN_CHANNEL_CLI").ok();
        let original_desktop = env::var("MORN_CHANNEL_DESKTOP").ok();
        let original_api = env::var("MORN_CHANNEL_REST_API").ok();
        let original_default = env::var("MORN_DEFAULT_CHANNEL").ok();

        env::set_var("MORN_CHANNEL_CLI", "false");
        env::set_var("MORN_CHANNEL_DESKTOP", "false");
        env::set_var("MORN_CHANNEL_REST_API", "true");
        env::set_var("MORN_DEFAULT_CHANNEL", "rest_api");

        let ch = ChannelsConfig::from_env();
        assert!(!ch.cli);
        assert!(!ch.desktop);
        assert!(ch.rest_api);
        assert_eq!(ch.default_channel, "rest_api");

        if let Some(v) = original_cli {
            env::set_var("MORN_CHANNEL_CLI", v);
        } else {
            env::remove_var("MORN_CHANNEL_CLI");
        }
        if let Some(v) = original_desktop {
            env::set_var("MORN_CHANNEL_DESKTOP", v);
        } else {
            env::remove_var("MORN_CHANNEL_DESKTOP");
        }
        if let Some(v) = original_api {
            env::set_var("MORN_CHANNEL_REST_API", v);
        } else {
            env::remove_var("MORN_CHANNEL_REST_API");
        }
        if let Some(v) = original_default {
            env::set_var("MORN_DEFAULT_CHANNEL", v);
        } else {
            env::remove_var("MORN_DEFAULT_CHANNEL");
        }
    }

    #[test]
    fn test_server_config_env_override() {
        let original_enabled = env::var("MORN_SERVER_ENABLED").ok();
        let original_host = env::var("MORN_SERVER_HOST").ok();
        let original_port = env::var("MORN_SERVER_PORT").ok();

        env::set_var("MORN_SERVER_ENABLED", "true");
        env::set_var("MORN_SERVER_HOST", "0.0.0.0");
        env::set_var("MORN_SERVER_PORT", "8080");

        let srv = ServerConfig::from_env();
        assert!(srv.enabled);
        assert_eq!(srv.host, "0.0.0.0");
        assert_eq!(srv.port, 8080);

        if let Some(v) = original_enabled {
            env::set_var("MORN_SERVER_ENABLED", v);
        } else {
            env::remove_var("MORN_SERVER_ENABLED");
        }
        if let Some(v) = original_host {
            env::set_var("MORN_SERVER_HOST", v);
        } else {
            env::remove_var("MORN_SERVER_HOST");
        }
        if let Some(v) = original_port {
            env::set_var("MORN_SERVER_PORT", v);
        } else {
            env::remove_var("MORN_SERVER_PORT");
        }
    }

    #[test]
    fn test_env_bool() {
        let original_1 = env::var("_TEST_BOOL_1").ok();
        let original_2 = env::var("_TEST_BOOL_2").ok();

        env::set_var("_TEST_BOOL_1", "true");
        assert!(env_bool("_TEST_BOOL_1", false));

        env::set_var("_TEST_BOOL_2", "false");
        assert!(!env_bool("_TEST_BOOL_2", true));

        assert_eq!(env_bool("_TEST_NONEXISTENT", true), true);

        if let Some(v) = original_1 {
            env::set_var("_TEST_BOOL_1", v);
        } else {
            env::remove_var("_TEST_BOOL_1");
        }
        if let Some(v) = original_2 {
            env::set_var("_TEST_BOOL_2", v);
        } else {
            env::remove_var("_TEST_BOOL_2");
        }
    }

    #[test]
    fn test_env_u16() {
        let original = env::var("_TEST_U16").ok();
        env::set_var("_TEST_U16", "8080");
        assert_eq!(env_u16("_TEST_U16", 0), 8080);
        if let Some(v) = original {
            env::set_var("_TEST_U16", v);
        } else {
            env::remove_var("_TEST_U16");
        }

        assert_eq!(env_u16("_TEST_NONEXISTENT_U16", 42), 42);
    }

    #[test]
    fn test_custom_provider_config_default() {
        let c = CustomProviderConfig::default();
        assert!(c.base_url.is_empty());
        assert!(c.api_key.is_none());
        assert!(c.models.is_empty());
        assert_eq!(c.api_key_header, "Authorization");
    }

    #[test]
    fn test_expand_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/test"), home.join("test"));
        assert_eq!(
            expand_tilde("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
    }
}
