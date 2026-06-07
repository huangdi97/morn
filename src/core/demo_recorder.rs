//! demo_recorder — Records demonstrations and interaction traces for later replay.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub action_type: String,
    pub target: String,
    pub input: Option<String>,
    pub timestamp: i64,
}

impl UserAction {
    pub fn new(action_type: &str, target: &str, input: Option<String>) -> Self {
        UserAction {
            action_type: action_type.to_string(),
            target: target.to_string(),
            input,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoRecorder {
    actions: Vec<UserAction>,
    recording: bool,
    replay_index: usize,
    replay_speed: f64,
}

impl DemoRecorder {
    pub fn new() -> Self {
        DemoRecorder {
            actions: Vec::new(),
            recording: false,
            replay_index: 0,
            replay_speed: 1.0,
        }
    }

    pub fn start_recording(&mut self) {
        self.actions.clear();
        self.recording = true;
        self.replay_index = 0;
    }

    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn record_action(&mut self, action: UserAction) {
        if self.recording {
            self.actions.push(action);
        }
    }

    pub fn record(&mut self, action_type: &str, target: &str, input: Option<String>) {
        if self.recording {
            self.actions
                .push(UserAction::new(action_type, target, input));
        }
    }

    pub fn actions(&self) -> &[UserAction] {
        &self.actions
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.replay_index = 0;
    }

    pub fn start_replay(&mut self) {
        self.replay_index = 0;
    }

    pub fn set_replay_speed(&mut self, speed: f64) {
        self.replay_speed = speed.max(0.1);
    }

    pub fn replay_speed(&self) -> f64 {
        self.replay_speed
    }

    pub fn next_action(&mut self) -> Option<&UserAction> {
        if self.replay_index < self.actions.len() {
            let action = &self.actions[self.replay_index];
            self.replay_index += 1;
            Some(action)
        } else {
            None
        }
    }

    pub fn has_next(&self) -> bool {
        self.replay_index < self.actions.len()
    }

    pub fn reset_replay(&mut self) {
        self.replay_index = 0;
    }

    pub fn current_replay_index(&self) -> usize {
        self.replay_index
    }

    pub fn total_actions(&self) -> usize {
        self.actions.len()
    }
}

impl Default for DemoRecorder {
    fn default() -> Self {
        DemoRecorder::new()
    }
}
