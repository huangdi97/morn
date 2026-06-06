use std::sync::Arc;

use crate::core::storage::Storage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnboardingStep {
    SelectLlm,
    ConfigureChannel,
    TestConversation,
    Completed,
}

impl OnboardingStep {
    fn as_str(&self) -> &'static str {
        match self {
            OnboardingStep::SelectLlm => "select_llm",
            OnboardingStep::ConfigureChannel => "configure_channel",
            OnboardingStep::TestConversation => "test_conversation",
            OnboardingStep::Completed => "completed",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "select_llm" => OnboardingStep::SelectLlm,
            "configure_channel" => OnboardingStep::ConfigureChannel,
            "test_conversation" => OnboardingStep::TestConversation,
            "completed" => OnboardingStep::Completed,
            _ => OnboardingStep::SelectLlm,
        }
    }
}

pub struct OnboardingWizard {
    storage: Arc<Storage>,
}

impl OnboardingWizard {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub fn is_first_run(&self) -> bool {
        self.storage
            .get_setting("onboarding_completed")
            .ok()
            .flatten()
            .map(|v| v != "true")
            .unwrap_or(true)
    }

    pub fn mark_completed(&self) -> Result<(), String> {
        self.storage.set_setting("onboarding_completed", "true")?;
        self.storage.set_setting("onboarding_step", "completed")?;
        Ok(())
    }

    pub fn get_current_step(&self) -> OnboardingStep {
        if !self.is_first_run() {
            return OnboardingStep::Completed;
        }
        let step = self
            .storage
            .get_setting("onboarding_step")
            .ok()
            .flatten()
            .unwrap_or_default();
        OnboardingStep::from_str(&step)
    }

    pub fn next_step(&self) -> Result<OnboardingStep, String> {
        let current = self.get_current_step();
        let next = match current {
            OnboardingStep::SelectLlm => OnboardingStep::ConfigureChannel,
            OnboardingStep::ConfigureChannel => OnboardingStep::TestConversation,
            OnboardingStep::TestConversation => {
                self.mark_completed()?;
                OnboardingStep::Completed
            }
            OnboardingStep::Completed => OnboardingStep::Completed,
        };
        if current != OnboardingStep::Completed {
            self.storage.set_setting("onboarding_step", next.as_str())?;
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    fn make_wizard() -> OnboardingWizard {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        OnboardingWizard::new(storage)
    }

    #[test]
    fn test_first_run_defaults_to_true() {
        let wizard = make_wizard();
        assert!(wizard.is_first_run());
    }

    #[test]
    fn test_mark_completed() {
        let wizard = make_wizard();
        assert!(wizard.is_first_run());
        wizard.mark_completed().unwrap();
        assert!(!wizard.is_first_run());
    }

    #[test]
    fn test_initial_step_is_select_llm() {
        let wizard = make_wizard();
        assert_eq!(wizard.get_current_step(), OnboardingStep::SelectLlm);
    }

    #[test]
    fn test_next_step_progression() {
        let wizard = make_wizard();
        assert_eq!(wizard.get_current_step(), OnboardingStep::SelectLlm);

        let step = wizard.next_step().unwrap();
        assert_eq!(step, OnboardingStep::ConfigureChannel);
        assert_eq!(wizard.get_current_step(), OnboardingStep::ConfigureChannel);

        let step = wizard.next_step().unwrap();
        assert_eq!(step, OnboardingStep::TestConversation);
        assert_eq!(wizard.get_current_step(), OnboardingStep::TestConversation);

        let step = wizard.next_step().unwrap();
        assert_eq!(step, OnboardingStep::Completed);
        assert_eq!(wizard.get_current_step(), OnboardingStep::Completed);
    }

    #[test]
    fn test_next_step_after_completed_stays_completed() {
        let wizard = make_wizard();
        wizard.mark_completed().unwrap();
        assert_eq!(wizard.get_current_step(), OnboardingStep::Completed);
        let step = wizard.next_step().unwrap();
        assert_eq!(step, OnboardingStep::Completed);
    }
}
