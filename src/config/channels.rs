use serde::Deserialize;
use std::env;
use std::path::PathBuf;

use super::{deserialize_path, env_bool, env_path, expand_tilde};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ChannelsConfig {
    pub cli: bool,
    pub desktop: bool,
    pub rest_api: bool,
    pub browser_ext: bool,
    pub telegram: TelegramConfig,
    pub wecom: WeComConfig,
    pub dingtalk: DingTalkConfig,
    pub pushplus: PushPlusConfig,
    pub serverchan: ServerChanConfig,
    pub miniprogram: MiniProgramConfig,
    pub default_channel: String,
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            cli: true,
            desktop: true,
            rest_api: false,
            browser_ext: false,
            telegram: TelegramConfig::default(),
            wecom: WeComConfig::default(),
            dingtalk: DingTalkConfig::default(),
            pushplus: PushPlusConfig::default(),
            serverchan: ServerChanConfig::default(),
            miniprogram: MiniProgramConfig::default(),
            default_channel: "cli".to_string(),
        }
    }
}

impl ChannelsConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            cli: env_bool("MORN_CHANNEL_CLI", default.cli),
            desktop: env_bool("MORN_CHANNEL_DESKTOP", default.desktop),
            rest_api: env_bool("MORN_CHANNEL_REST_API", default.rest_api),
            browser_ext: env_bool("MORN_CHANNEL_BROWSER_EXT", default.browser_ext),
            telegram: TelegramConfig::from_env(),
            wecom: WeComConfig::from_env(),
            dingtalk: DingTalkConfig::from_env(),
            pushplus: PushPlusConfig::from_env(),
            serverchan: ServerChanConfig::from_env(),
            miniprogram: MiniProgramConfig::from_env(),
            default_channel: env::var("MORN_DEFAULT_CHANNEL").unwrap_or(default.default_channel),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub chat_id: String,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: None,
            chat_id: "0".to_string(),
        }
    }
}

impl TelegramConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_TELEGRAM_ENABLED", default.enabled),
            bot_token: env::var("TELEGRAM_BOT_TOKEN").ok(),
            chat_id: env::var("TELEGRAM_CHAT_ID").unwrap_or(default.chat_id),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct WeComConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub token: Option<String>,
    pub encoding_aes_key: Option<String>,
    pub corp_id: Option<String>,
}

impl WeComConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_WECOM_ENABLED", default.enabled),
            webhook_url: env::var("MORN_CHANNEL_WECOM_WEBHOOK_URL")
                .or_else(|_| env::var("WECOM_WEBHOOK_URL"))
                .ok(),
            token: env::var("MORN_CHANNEL_WECOM_TOKEN")
                .or_else(|_| env::var("WECOM_TOKEN"))
                .ok(),
            encoding_aes_key: env::var("MORN_CHANNEL_WECOM_ENCODING_AES_KEY")
                .or_else(|_| env::var("WECOM_ENCODING_AES_KEY"))
                .ok(),
            corp_id: env::var("MORN_CHANNEL_WECOM_CORP_ID")
                .or_else(|_| env::var("WECOM_CORP_ID"))
                .ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct DingTalkConfig {
    pub enabled: bool,
    pub webhook_token: Option<String>,
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
}

impl DingTalkConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_DINGTALK_ENABLED", default.enabled),
            webhook_token: env::var("MORN_CHANNEL_DINGTALK_WEBHOOK_TOKEN")
                .or_else(|_| env::var("DINGTALK_WEBHOOK_TOKEN"))
                .ok(),
            app_key: env::var("MORN_CHANNEL_DINGTALK_APP_KEY")
                .or_else(|_| env::var("DINGTALK_APP_KEY"))
                .ok(),
            app_secret: env::var("MORN_CHANNEL_DINGTALK_APP_SECRET")
                .or_else(|_| env::var("DINGTALK_APP_SECRET"))
                .ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PushPlusConfig {
    pub enabled: bool,
    pub token: Option<String>,
}

impl PushPlusConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_PUSHPLUS_ENABLED", default.enabled),
            token: env::var("MORN_CHANNEL_PUSHPLUS_TOKEN")
                .or_else(|_| env::var("PUSHPLUS_TOKEN"))
                .ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ServerChanConfig {
    pub enabled: bool,
    pub token: Option<String>,
}

impl ServerChanConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_SERVERCHAN_ENABLED", default.enabled),
            token: env::var("MORN_CHANNEL_SERVERCHAN_TOKEN")
                .or_else(|_| env::var("SERVERCHAN_TOKEN"))
                .ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct MiniProgramConfig {
    pub enabled: bool,
    pub appid: Option<String>,
    pub secret: Option<String>,
    pub token: Option<String>,
}

impl MiniProgramConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_CHANNEL_MINIPROGRAM_ENABLED", default.enabled),
            appid: env::var("MORN_CHANNEL_MINIPROGRAM_APPID")
                .or_else(|_| env::var("MINIPROGRAM_APPID"))
                .ok(),
            secret: env::var("MORN_CHANNEL_MINIPROGRAM_SECRET")
                .or_else(|_| env::var("MINIPROGRAM_SECRET"))
                .ok(),
            token: env::var("MORN_CHANNEL_MINIPROGRAM_TOKEN")
                .or_else(|_| env::var("MINIPROGRAM_TOKEN"))
                .ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "deserialize_path")]
    pub data_dir: PathBuf,
    #[serde(deserialize_with = "deserialize_path")]
    pub log_dir: PathBuf,
    #[serde(deserialize_with = "deserialize_path")]
    pub pid_file: PathBuf,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            data_dir: expand_tilde("~/.local/share/morn"),
            log_dir: expand_tilde("~/.local/state/morn/logs"),
            pid_file: expand_tilde("~/.local/state/morn/morn.pid"),
        }
    }
}

impl DaemonConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            enabled: env_bool("MORN_DAEMON_ENABLED", default.enabled),
            data_dir: env_path("MORN_DAEMON_DATA_DIR", default.data_dir),
            log_dir: env_path("MORN_DAEMON_LOG_DIR", default.log_dir),
            pid_file: env_path("MORN_DAEMON_PID_FILE", default.pid_file),
        }
    }
}
