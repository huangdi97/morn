use serde_yaml::Value;
use std::fs;
use std::path::PathBuf;

pub struct Onboarding {
    completed: bool,
    current_step: OnboardingStep,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnboardingStep {
    Welcome,
    SelectLLM,
    ConfigureChannel,
    TestConversation,
    Done,
}

impl Onboarding {
    pub fn new() -> Self {
        Self {
            completed: !Self::is_first_run(),
            current_step: OnboardingStep::Welcome,
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

        let Ok(config) = serde_yaml::from_str::<Value>(&config) else {
            return true;
        };

        config
            .get("first_run")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn steps() -> Vec<OnboardingStep> {
        vec![
            OnboardingStep::Welcome,
            OnboardingStep::SelectLLM,
            OnboardingStep::ConfigureChannel,
            OnboardingStep::TestConversation,
            OnboardingStep::Done,
        ]
    }

    pub fn current_step(&self) -> OnboardingStep {
        self.current_step
    }

    pub fn advance(&mut self, next_step: OnboardingStep) {
        self.current_step = next_step;
    }

    pub fn complete(&mut self) {
        if !self.completed {
            self.completed = true;
        }

        self.current_step = OnboardingStep::Done;
        let _ = persist_first_run(false);
    }
}

impl Default for Onboarding {
    fn default() -> Self {
        Self::new()
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".morn")
        .join("config.yaml")
}

fn persist_first_run(first_run: bool) -> std::io::Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut config = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_yaml::from_str::<Value>(&content).ok())
            .unwrap_or(Value::Mapping(Default::default()))
    } else {
        Value::Mapping(Default::default())
    };

    if !config.is_mapping() {
        config = Value::Mapping(Default::default());
    }

    if let Some(mapping) = config.as_mapping_mut() {
        mapping.insert(
            Value::String("first_run".to_string()),
            Value::Bool(first_run),
        );
    }

    let content =
        serde_yaml::to_string(&config).unwrap_or_else(|_| "first_run: false\n".to_string());
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static HOME_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_temp_home(test: impl FnOnce(&PathBuf)) {
        let _guard = HOME_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("home lock should not be poisoned");
        let original_home = env::var_os("HOME");
        let temp_home = tempdir().expect("temp home should be created");

        env::set_var("HOME", temp_home.path());
        test(&temp_home.path().to_path_buf());

        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn detects_first_run_when_config_is_missing() {
        with_temp_home(|_| {
            assert!(Onboarding::is_first_run());
        });
    }

    #[test]
    fn progresses_to_next_step() {
        with_temp_home(|_| {
            let mut onboarding = Onboarding::new();

            onboarding.advance(OnboardingStep::SelectLLM);

            assert_eq!(onboarding.current_step(), OnboardingStep::SelectLLM);
        });
    }

    #[test]
    fn complete_marks_onboarding_done() {
        with_temp_home(|home| {
            let mut onboarding = Onboarding::new();

            onboarding.complete();

            assert!(onboarding.completed);
            assert_eq!(onboarding.current_step(), OnboardingStep::Done);
            assert!(!Onboarding::is_first_run());
            assert!(home.join(".morn").join("config.yaml").exists());
        });
    }

    #[test]
    fn returns_steps_in_order() {
        assert_eq!(
            Onboarding::steps(),
            vec![
                OnboardingStep::Welcome,
                OnboardingStep::SelectLLM,
                OnboardingStep::ConfigureChannel,
                OnboardingStep::TestConversation,
                OnboardingStep::Done,
            ]
        );
    }

    #[test]
    fn default_state_is_welcome() {
        with_temp_home(|_| {
            let onboarding = Onboarding::new();

            assert_eq!(onboarding.current_step(), OnboardingStep::Welcome);
        });
    }
}
