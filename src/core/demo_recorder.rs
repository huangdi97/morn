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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_not_recording() {
        let recorder = DemoRecorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.action_count(), 0);
        assert!(!recorder.has_next());
    }

    #[test]
    fn test_start_recording_clears_actions() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("click", "#btn", Some("data".into()));
        assert_eq!(recorder.action_count(), 1);

        recorder.start_recording();
        assert_eq!(recorder.action_count(), 0);
        assert!(recorder.is_recording());
    }

    #[test]
    fn test_stop_recording() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        assert!(recorder.is_recording());
        recorder.stop_recording();
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_record_ignored_when_not_recording() {
        let mut recorder = DemoRecorder::new();
        recorder.record("click", "#btn", Some("data".into()));
        assert_eq!(recorder.action_count(), 0);
    }

    #[test]
    fn test_record_action_when_recording() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        let action = UserAction::new("click", "#btn", Some("data".into()));
        recorder.record_action(action);
        assert_eq!(recorder.action_count(), 1);
    }

    #[test]
    fn test_record_convenience_method() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("type", "#input", Some("hello".into()));
        recorder.record("click", "#submit", None);
        assert_eq!(recorder.action_count(), 2);
    }

    #[test]
    fn test_actions_returns_slice() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("click", "#btn", None);
        let actions = recorder.actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "click");
    }

    #[test]
    fn test_clear() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("click", "#btn", None);
        recorder.clear();
        assert_eq!(recorder.action_count(), 0);
        assert_eq!(recorder.current_replay_index(), 0);
    }

    #[test]
    fn test_replay_lifecycle() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("click", "#btn", None);
        recorder.record("type", "#input", Some("text".into()));
        recorder.stop_recording();

        recorder.start_replay();
        assert!(recorder.has_next());

        let first = recorder.next_action().unwrap();
        assert_eq!(first.action_type, "click");

        let second = recorder.next_action().unwrap();
        assert_eq!(second.action_type, "type");

        assert!(!recorder.has_next());
        assert!(recorder.next_action().is_none());
    }

    #[test]
    fn test_reset_replay() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("click", "#btn", None);
        recorder.stop_recording();

        recorder.start_replay();
        recorder.next_action();
        assert_eq!(recorder.current_replay_index(), 1);

        recorder.reset_replay();
        assert_eq!(recorder.current_replay_index(), 0);
    }

    #[test]
    fn test_set_replay_speed_clamps_low() {
        let mut recorder = DemoRecorder::new();
        recorder.set_replay_speed(0.01);
        assert!((recorder.replay_speed() - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_replay_speed_normal() {
        let mut recorder = DemoRecorder::new();
        recorder.set_replay_speed(2.0);
        assert!((recorder.replay_speed() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_total_actions() {
        let mut recorder = DemoRecorder::new();
        recorder.start_recording();
        recorder.record("a", "t1", None);
        recorder.record("b", "t2", None);
        recorder.record("c", "t3", None);
        assert_eq!(recorder.total_actions(), 3);
        assert_eq!(recorder.action_count(), recorder.total_actions());
    }

    #[test]
    fn test_default_trait() {
        let recorder = DemoRecorder::default();
        assert!(!recorder.is_recording());
    }
}
