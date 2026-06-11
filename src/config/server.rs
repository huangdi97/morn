use serde::Deserialize;
use std::env;

use super::{env_bool, env_u16};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 3030,
            base_url: "http://127.0.0.1:3030".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_SERVER_ENABLED", default.enabled),
            host: env::var("MORN_SERVER_HOST").unwrap_or(default.host),
            port: env_u16("MORN_SERVER_PORT", default.port),
            base_url: env::var("MORN_SERVER_BASE_URL").unwrap_or(default.base_url),
        }
    }
}
