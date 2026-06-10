use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct TimedTrigger {
    pub id: String,
    pub interval_secs: u64,
    pub last_fired: Option<SystemTime>,
    pub handler: &'static str,
}

impl TimedTrigger {
    pub fn new(id: &str, interval_secs: u64, handler: &'static str) -> Self {
        TimedTrigger {
            id: id.to_string(),
            interval_secs,
            last_fired: None,
            handler,
        }
    }

    pub fn is_ready(&self, now: SystemTime) -> bool {
        match self.last_fired {
            Some(last) => {
                now.duration_since(last)
                    .map(|d| d.as_secs() >= self.interval_secs)
                    .unwrap_or(true)
            }
            None => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventTrigger {
    pub event_type: String,
    pub handler: &'static str,
    pub enabled: bool,
}

impl EventTrigger {
    pub fn new(event_type: &str, handler: &'static str) -> Self {
        EventTrigger {
            event_type: event_type.to_string(),
            handler,
            enabled: true,
        }
    }
}