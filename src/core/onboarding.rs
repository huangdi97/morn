//! First-run onboarding wizard and setup flow.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml::Value;

pub struct Onboarding {
    completed: bool,
    current_step: OnboardingStep,
    draft: OnboardingDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnboardingStep {
    Welcome,
    ConfigureApiKey,
    ConfigureChannel,
    SelectModel,
    Done,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct OnboardingDraft {
    pub api_key: Option<String>,
    pub channel: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

impl Onboarding {
    pub fn new() -> Self {
        Self {
            completed: !Self::is_first_run(),
            current_step: OnboardingStep::Welcome,
            draft: OnboardingDraft::default(),
        }
    }

    pub fn is_first_run() -> bool {
        let config_path = config_path();

        if !config_path.exists() {
            return true;
        }

        let Ok(config) = fs::read_to_string(config_path) else {
            return true;
        };

        let Ok(config) = config.parse::<Value>() else {
            return true;
        };

        config
            .get("onboarding")
            .and_then(|v| v.get("completed"))
            .and_then(Value::as_bool)
            .map(|completed| !completed)
            .unwrap_or(true)
    }

    pub fn steps() -> Vec<OnboardingStep> {
        vec![
            OnboardingStep::Welcome,
            OnboardingStep::ConfigureApiKey,
            OnboardingStep::ConfigureChannel,
            OnboardingStep::SelectModel,
            OnboardingStep::Done,
        ]
    }

    pub fn current_step(&self) -> OnboardingStep {
        self.current_step
    }

    pub fn advance(&mut self, next_step: OnboardingStep) {
        self.current_step = next_step;
    }

    pub fn set_api_key(&mut self, api_key: impl Into<String>) -> Result<(), String> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        self.draft.api_key = Some(api_key);
        self.current_step = OnboardingStep::ConfigureChannel;
        Ok(())
    }

    pub fn select_channel(&mut self, channel: impl Into<String>) -> Result<(), String> {
        let channel = channel.into();
        if !is_supported_channel(&channel) {
            return Err(format!("Unsupported channel: {}", channel));
        }
        self.draft.channel = Some(channel);
        self.current_step = OnboardingStep::SelectModel;
        Ok(())
    }

    pub fn select_model(
        &mut self,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<(), String> {
        let provider = provider.into();
        let model = model.into();
        if provider.trim().is_empty() || model.trim().is_empty() {
            return Err("Provider and model cannot be empty".to_string());
        }
        self.draft.provider = Some(provider);
        self.draft.model = Some(model);
        Ok(())
    }

    pub fn draft(&self) -> &OnboardingDraft {
        &self.draft
    }

    pub fn complete(&mut self) -> Result<(), String> {
        self.validate()?;
        if !self.completed {
            self.completed = true;
        }

        self.current_step = OnboardingStep::Done;
        persist_onboarding_config(&self.draft)?;
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self
            .draft
            .api_key
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err("API key is required before completing onboarding".to_string());
        }
        if self
            .draft
            .channel
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err("Channel selection is required before completing onboarding".to_string());
        }
        if self
            .draft
            .provider
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
            || self.draft.model.as_deref().unwrap_or("").trim().is_empty()
        {
            return Err("Model selection is required before completing onboarding".to_string());
        }
        Ok(())
    }
}

impl Default for Onboarding {
    fn default() -> Self {
        Self::new()
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("morn").join("config.toml")
}

fn persist_onboarding_config(draft: &OnboardingDraft) -> Result<(), String> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create config directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    #[derive(Serialize)]
    struct PersistedConfig {
        onboarding: OnboardingField,
        model: ModelField,
        channels: ChannelsField,
    }
    #[derive(Serialize)]
    struct OnboardingField {
        completed: bool,
    }
    #[derive(Serialize)]
    struct ModelField {
        provider: String,
        name: String,
        api_key: String,
    }
    #[derive(Serialize)]
    struct ChannelsField {
        default_channel: String,
        cli: bool,
        desktop: bool,
        rest_api: bool,
        browser_ext: bool,
    }

    let channel = draft.channel.clone().unwrap_or_else(|| "cli".to_string());
    let persisted = PersistedConfig {
        onboarding: OnboardingField { completed: true },
        model: ModelField {
            provider: draft.provider.clone().unwrap_or_else(|| "deepseek".to_string()),
            name: draft.model.clone().unwrap_or_else(|| "deepseek-chat".to_string()),
            api_key: draft.api_key.clone().unwrap_or_default(),
        },
        channels: ChannelsField {
            default_channel: channel.clone(),
            cli: channel == "cli",
            desktop: channel == "desktop",
            rest_api: channel == "rest_api",
            browser_ext: channel == "browser_ext",
        },
    };

    let content = toml::to_string_pretty(&persisted)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, &content).map_err(|e| format!("Failed to write onboarding config: {}", e))?;

    fs::metadata(&path).map_err(|e| format!("Failed to verify config write: {}", e))?;

    Ok(())
}

fn is_supported_channel(channel: &str) -> bool {
    matches!(
        channel,
        "cli"
            | "desktop"
            | "rest_api"
            | "browser_ext"
            | "telegram"
            | "wecom"
            | "dingtalk"
            | "miniprogram"
            | "pushplus"
            | "serverchan"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static HOME_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_temp_home(test: impl FnOnce()) {
        let _guard = HOME_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| panic!("home lock should not be poisoned: {}", e));
        let original_home = env::var_os("HOME");
        let temp_home = tempdir().unwrap_or_else(|e| panic!("temp home should be created: {}", e));

        env::set_var("HOME", temp_home.path());
        test();

        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn detects_first_run_when_config_is_missing() {
        with_temp_home(|| {
            assert!(Onboarding::is_first_run());
            assert!(!config_path().exists());
        });
    }

    #[test]
    fn progresses_to_next_step() {
        with_temp_home(|| {
            let mut onboarding = Onboarding::new();

            onboarding.advance(OnboardingStep::ConfigureApiKey);

            assert_eq!(onboarding.current_step(), OnboardingStep::ConfigureApiKey);
        });
    }

    #[test]
    fn complete_marks_onboarding_done() {
        with_temp_home(|| {
            let mut onboarding = Onboarding::new();

            onboarding.set_api_key("sk-test").unwrap();
            onboarding.select_channel("cli").unwrap();
            onboarding
                .select_model("deepseek", "deepseek-chat")
                .unwrap();
            onboarding.complete().unwrap();

            assert!(onboarding.completed);
            assert_eq!(onboarding.current_step(), OnboardingStep::Done);
            assert!(!Onboarding::is_first_run());
            assert!(config_path().exists());
        });
    }

    #[test]
    fn returns_steps_in_order() {
        assert_eq!(
            Onboarding::steps(),
            vec![
                OnboardingStep::Welcome,
                OnboardingStep::ConfigureApiKey,
                OnboardingStep::ConfigureChannel,
                OnboardingStep::SelectModel,
                OnboardingStep::Done,
            ]
        );
    }

    #[test]
    fn wizard_records_api_channel_and_model() {
        with_temp_home(|| {
            let mut onboarding = Onboarding::new();

            onboarding.set_api_key("sk-test").unwrap();
            assert_eq!(onboarding.current_step(), OnboardingStep::ConfigureChannel);
            onboarding.select_channel("desktop").unwrap();
            assert_eq!(onboarding.current_step(), OnboardingStep::SelectModel);
            onboarding.select_model("openai", "gpt-4o-mini").unwrap();

            assert_eq!(
                onboarding.draft(),
                &OnboardingDraft {
                    api_key: Some("sk-test".to_string()),
                    channel: Some("desktop".to_string()),
                    provider: Some("openai".to_string()),
                    model: Some("gpt-4o-mini".to_string()),
                }
            );
        });
    }

    #[test]
    fn default_state_is_welcome() {
        with_temp_home(|| {
            let onboarding = Onboarding::new();

            assert_eq!(onboarding.current_step(), OnboardingStep::Welcome);
            assert!(!config_path().exists());
        });
    }
}
