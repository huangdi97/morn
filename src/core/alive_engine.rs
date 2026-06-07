//! alive_engine — Generates lightweight presence prompts from time and task events.

use crate::core::event_bus::{Event, SimpleEventBus, EVENT_TASK_COMPLETED, EVENT_TASK_FAILED};

#[derive(Debug, Clone)]
pub struct AliveEngine {
    config: AliveEngineConfig,
    last_interaction: Option<i64>,
    daily_greeting_sent: bool,
    daily_greeting_count: u32,
}

#[derive(Debug, Clone)]
pub struct AliveEngineConfig {
    pub enabled: bool,
    pub greeting_style: GreetingStyle,
    pub idle_threshold_minutes: u64,
    pub daily_max_greetings: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreetingStyle {
    Casual,
    Professional,
    Warm,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliveTrigger {
    Idle,
    TaskComplete,
    TaskFailed,
    Morning,
    Evening,
}

impl AliveEngine {
    pub fn new(config: AliveEngineConfig) -> Self {
        Self {
            config,
            last_interaction: Some(Self::now_timestamp()),
            daily_greeting_sent: false,
            daily_greeting_count: 0,
        }
    }

    pub fn should_trigger(&self, trigger: &AliveTrigger) -> bool {
        if !self.config.enabled || self.daily_greeting_count >= self.config.daily_max_greetings {
            return false;
        }

        match trigger {
            AliveTrigger::Idle => self.get_idle_minutes() >= self.config.idle_threshold_minutes,
            AliveTrigger::TaskComplete | AliveTrigger::TaskFailed => true,
            AliveTrigger::Morning | AliveTrigger::Evening => !self.daily_greeting_sent,
        }
    }

    pub fn generate_greeting(&self, trigger: &AliveTrigger) -> String {
        match (self.config.greeting_style, trigger) {
            (GreetingStyle::Casual, AliveTrigger::Idle) => {
                "Still here if you want to keep going.".to_string()
            }
            (GreetingStyle::Casual, AliveTrigger::TaskComplete) => {
                "Nice, that task is done.".to_string()
            }
            (GreetingStyle::Casual, AliveTrigger::TaskFailed) => {
                "That task hit an issue. Want me to take another look?".to_string()
            }
            (GreetingStyle::Casual, AliveTrigger::Morning) => {
                "Morning. Ready when you are.".to_string()
            }
            (GreetingStyle::Casual, AliveTrigger::Evening) => {
                "Evening. I can help wrap things up.".to_string()
            }
            (GreetingStyle::Professional, AliveTrigger::Idle) => {
                "I am available if you would like to continue.".to_string()
            }
            (GreetingStyle::Professional, AliveTrigger::TaskComplete) => {
                "The task has completed successfully.".to_string()
            }
            (GreetingStyle::Professional, AliveTrigger::TaskFailed) => {
                "The task failed. I can review the error and propose next steps.".to_string()
            }
            (GreetingStyle::Professional, AliveTrigger::Morning) => {
                "Good morning. I am ready to assist.".to_string()
            }
            (GreetingStyle::Professional, AliveTrigger::Evening) => {
                "Good evening. I can help finish any remaining work.".to_string()
            }
            (GreetingStyle::Warm, AliveTrigger::Idle) => {
                "I am still here whenever you are ready.".to_string()
            }
            (GreetingStyle::Warm, AliveTrigger::TaskComplete) => {
                "That is complete. Good progress.".to_string()
            }
            (GreetingStyle::Warm, AliveTrigger::TaskFailed) => {
                "That did not land cleanly. I can help sort it out.".to_string()
            }
            (GreetingStyle::Warm, AliveTrigger::Morning) => {
                "Good morning. I hope your day is starting well.".to_string()
            }
            (GreetingStyle::Warm, AliveTrigger::Evening) => {
                "Good evening. I am here if you want to close anything out.".to_string()
            }
            (GreetingStyle::Minimal, AliveTrigger::Idle) => "Ready.".to_string(),
            (GreetingStyle::Minimal, AliveTrigger::TaskComplete) => "Complete.".to_string(),
            (GreetingStyle::Minimal, AliveTrigger::TaskFailed) => "Failed.".to_string(),
            (GreetingStyle::Minimal, AliveTrigger::Morning) => "Morning.".to_string(),
            (GreetingStyle::Minimal, AliveTrigger::Evening) => "Evening.".to_string(),
        }
    }

    pub fn on_tick(&mut self) -> Option<String> {
        if self.should_trigger(&AliveTrigger::Idle) {
            let greeting = self.generate_greeting(&AliveTrigger::Idle);
            self.mark_greeting_sent();
            Some(greeting)
        } else {
            None
        }
    }

    pub fn reset_daily_counters(&mut self) {
        self.daily_greeting_sent = false;
        self.daily_greeting_count = 0;
    }

    pub fn record_interaction(&mut self) {
        self.last_interaction = Some(Self::now_timestamp());
    }

    pub fn get_idle_minutes(&self) -> u64 {
        self.last_interaction
            .map(|last_interaction| {
                let elapsed_seconds = Self::now_timestamp().saturating_sub(last_interaction);
                (elapsed_seconds as u64) / 60
            })
            .unwrap_or(0)
    }

    pub fn on_event(&mut self, event: &Event) -> Option<String> {
        let trigger = Self::trigger_from_event(event)?;

        if self.should_trigger(&trigger) {
            let greeting = self.generate_greeting(&trigger);
            self.mark_greeting_sent();
            Some(greeting)
        } else {
            None
        }
    }

    pub fn trigger_from_event(event: &Event) -> Option<AliveTrigger> {
        match event.event_type.as_str() {
            EVENT_TASK_COMPLETED => Some(AliveTrigger::TaskComplete),
            EVENT_TASK_FAILED => Some(AliveTrigger::TaskFailed),
            _ => None,
        }
    }

    pub fn register_event_hooks(bus: &mut SimpleEventBus, handler: fn(Event)) {
        bus.subscribe(EVENT_TASK_COMPLETED, handler);
        bus.subscribe(EVENT_TASK_FAILED, handler);
    }

    fn mark_greeting_sent(&mut self) {
        self.daily_greeting_sent = true;
        self.daily_greeting_count = self.daily_greeting_count.saturating_add(1);
    }

    fn now_timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }
}

impl Default for AliveEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            greeting_style: GreetingStyle::Casual,
            idle_threshold_minutes: 30,
            daily_max_greetings: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_default_config() -> AliveEngine {
        AliveEngine::new(AliveEngineConfig::default())
    }

    #[test]
    fn should_trigger_on_idle_after_threshold() {
        let mut engine = engine_with_default_config();
        engine.last_interaction = Some(AliveEngine::now_timestamp() - 31 * 60);

        assert!(engine.should_trigger(&AliveTrigger::Idle));
    }

    #[test]
    fn generate_greeting_returns_non_empty_string_for_all_trigger_types() {
        let engine = engine_with_default_config();
        let triggers = [
            AliveTrigger::Idle,
            AliveTrigger::TaskComplete,
            AliveTrigger::TaskFailed,
            AliveTrigger::Morning,
            AliveTrigger::Evening,
        ];

        for trigger in triggers {
            assert!(!engine.generate_greeting(&trigger).is_empty());
        }
    }

    #[test]
    fn record_interaction_resets_idle_timer() {
        let mut engine = engine_with_default_config();
        engine.last_interaction = Some(AliveEngine::now_timestamp() - 60 * 60);

        assert!(engine.get_idle_minutes() >= 60);

        engine.record_interaction();

        assert_eq!(engine.get_idle_minutes(), 0);
    }

    #[test]
    fn daily_counter_reset_works() {
        let mut engine = engine_with_default_config();
        engine.mark_greeting_sent();

        assert!(engine.daily_greeting_sent);
        assert_eq!(engine.daily_greeting_count, 1);

        engine.reset_daily_counters();

        assert!(!engine.daily_greeting_sent);
        assert_eq!(engine.daily_greeting_count, 0);
    }
}
