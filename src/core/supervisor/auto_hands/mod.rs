//! 自动执行模块 — 自主触发与预审批的自动操作
use crate::core::event_bus::SimpleEventBus;
use std::collections::VecDeque;
use std::time::SystemTime;

pub mod handlers;
pub mod triggers;

pub use triggers::{EventTrigger, TimedTrigger};

#[derive(Debug, Clone, PartialEq)]
pub enum MaintenanceTask {
    Inspection,
    GarbageCollection,
    MemoryCompaction,
    LogRotation,
    HealthCheck,
    CacheCleanup,
}

impl MaintenanceTask {
    pub fn label(&self) -> &'static str {
        match self {
            MaintenanceTask::Inspection => "inspection",
            MaintenanceTask::GarbageCollection => "gc",
            MaintenanceTask::MemoryCompaction => "memory_compaction",
            MaintenanceTask::LogRotation => "log_rotation",
            MaintenanceTask::HealthCheck => "health_check",
            MaintenanceTask::CacheCleanup => "cache_cleanup",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            MaintenanceTask::HealthCheck => 1,
            MaintenanceTask::GarbageCollection => 2,
            MaintenanceTask::Inspection => 3,
            MaintenanceTask::MemoryCompaction => 4,
            MaintenanceTask::CacheCleanup => 5,
            MaintenanceTask::LogRotation => 6,
        }
    }
}

pub struct AutoHands {
    pub enabled: bool,
    pub timed_triggers: Vec<TimedTrigger>,
    pub event_triggers: Vec<EventTrigger>,
    maintenance_queue: VecDeque<MaintenanceTask>,
    pub execution_count: u64,
    pub last_run: Option<SystemTime>,
    pub event_bus: Option<SimpleEventBus>,
}

impl AutoHands {
    pub fn new(event_bus: Option<SimpleEventBus>) -> Self {
        AutoHands {
            enabled: true,
            timed_triggers: vec![
                TimedTrigger::new("health-every-60s", 60, "health_check"),
                TimedTrigger::new("gc-every-300s", 300, "gc"),
                TimedTrigger::new("inspect-every-600s", 600, "inspection"),
                TimedTrigger::new("compact-every-1800s", 1800, "memory_compaction"),
            ],
            event_triggers: vec![
                EventTrigger::new("system.ready", "health_check"),
                EventTrigger::new("agent.created", "inspection"),
                EventTrigger::new("agent.destroyed", "gc"),
                EventTrigger::new("system.shutdown", "log_rotation"),
            ],
            maintenance_queue: VecDeque::new(),
            execution_count: 0,
            last_run: None,
            event_bus,
        }
    }

    pub fn enqueue(&mut self, task: MaintenanceTask) {
        self.maintenance_queue.push_back(task);
    }

    pub fn enqueue_priority(&mut self, task: MaintenanceTask) {
        self.maintenance_queue.push_front(task);
    }

    pub fn queue_len(&self) -> usize {
        self.maintenance_queue.len()
    }

    pub fn drain_queue(&mut self) -> Vec<MaintenanceTask> {
        self.maintenance_queue.drain(..).collect()
    }

    pub fn peek_queue(&self) -> Vec<&MaintenanceTask> {
        self.maintenance_queue.iter().collect()
    }

    pub fn add_timed_trigger(&mut self, trigger: TimedTrigger) {
        self.timed_triggers.push(trigger);
    }

    pub fn add_event_trigger(&mut self, trigger: EventTrigger) {
        self.event_triggers.push(trigger);
    }

    pub fn remove_timed_trigger(&mut self, id: &str) {
        self.timed_triggers.retain(|t| t.id != id);
    }
}

impl Default for AutoHands {
    fn default() -> Self {
        Self::new(None)
    }
}

fn timestamp() -> String {
    chrono::Utc::now().format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event_bus::Event;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::time::Duration;

    static EVENT_CALLS: AtomicUsize = AtomicUsize::new(0);
    fn event_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
    fn reset() {
        EVENT_CALLS.store(0, Ordering::SeqCst);
    }
    fn handler(_e: Event) {
        EVENT_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn test_new_auto_hands_has_default_triggers() {
        let ah = AutoHands::new(None);
        assert!(ah.enabled);
        assert_eq!(ah.timed_triggers.len(), 4);
        assert_eq!(ah.event_triggers.len(), 4);
    }

    #[test]
    fn test_enqueue_and_drain() {
        let mut ah = AutoHands::new(None);
        ah.enqueue(MaintenanceTask::HealthCheck);
        ah.enqueue(MaintenanceTask::GarbageCollection);
        assert_eq!(ah.queue_len(), 2);
        let drained = ah.drain_queue();
        assert_eq!(drained.len(), 2);
        assert_eq!(ah.queue_len(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut ah = AutoHands::new(None);
        ah.enqueue(MaintenanceTask::LogRotation);
        ah.enqueue(MaintenanceTask::HealthCheck);
        let drained = ah.drain_queue();
        assert_eq!(drained[0], MaintenanceTask::LogRotation);
    }

    #[test]
    fn test_enqueue_priority_front() {
        let mut ah = AutoHands::new(None);
        ah.enqueue(MaintenanceTask::LogRotation);
        ah.enqueue_priority(MaintenanceTask::HealthCheck);
        let drained = ah.drain_queue();
        assert_eq!(drained[0], MaintenanceTask::HealthCheck);
        assert_eq!(drained.len(), 2);
    }

    #[test]
    fn test_tick_fires_timed_triggers() {
        let mut ah = AutoHands::new(None);
        ah.timed_triggers.iter_mut().for_each(|t| {
            t.last_fired = Some(SystemTime::now() - Duration::from_secs(t.interval_secs + 1));
        });
        let fired = ah.tick();
        assert_eq!(fired.len(), 4);
    }

    #[test]
    fn test_tick_does_not_fire_before_interval() {
        let mut ah = AutoHands::new(None);
        ah.timed_triggers.iter_mut().for_each(|t| {
            t.last_fired = Some(SystemTime::now());
        });
        let fired = ah.tick();
        assert!(fired.is_empty());
    }

    #[test]
    fn test_handle_event_enqueues_task() {
        let mut ah = AutoHands::new(None);
        let event = Event::new("system.ready", "test", serde_json::json!({}));
        ah.handle_event(&event);
        assert_eq!(ah.queue_len(), 1);
        if let Some(task) = ah.drain_queue().first() {
            assert_eq!(*task, MaintenanceTask::HealthCheck);
        } else {
            panic!("expected a HealthCheck task");
        }
    }

    #[test]
    fn test_handle_event_unknown_type_does_nothing() {
        let mut ah = AutoHands::new(None);
        let event = Event::new("unknown.event", "test", serde_json::json!({}));
        ah.handle_event(&event);
        assert_eq!(ah.queue_len(), 0);
    }

    #[test]
    fn test_execute_task_returns_ok() {
        let ah = AutoHands::new(None);
        let result = ah.execute_task(&MaintenanceTask::HealthCheck);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("health check"));
    }

    #[test]
    fn test_disabled_auto_hands_does_not_tick() {
        let mut ah = AutoHands::new(None);
        ah.enabled = false;
        ah.timed_triggers.iter_mut().for_each(|t| {
            t.last_fired = Some(SystemTime::now() - Duration::from_secs(t.interval_secs + 1));
        });
        let fired = ah.tick();
        assert!(fired.is_empty());
    }

    #[test]
    fn test_disabled_auto_hands_does_not_handle_events() {
        let mut ah = AutoHands::new(None);
        ah.enabled = false;
        let event = Event::new("system.ready", "test", serde_json::json!({}));
        ah.handle_event(&event);
        assert_eq!(ah.queue_len(), 0);
    }

    #[test]
    fn test_run_executes_tasks_in_priority_order() {
        let mut ah = AutoHands::new(None);
        ah.timed_triggers.iter_mut().for_each(|t| {
            t.last_fired = Some(SystemTime::now());
        });
        ah.enqueue(MaintenanceTask::LogRotation);
        ah.enqueue(MaintenanceTask::HealthCheck);
        let results = ah.run();
        let health_idx = results
            .iter()
            .position(|(t, _)| *t == MaintenanceTask::HealthCheck);
        let log_idx = results
            .iter()
            .position(|(t, _)| *t == MaintenanceTask::LogRotation);
        assert!(health_idx.is_some());
        assert!(log_idx.is_some());
        assert!(health_idx.unwrap() < log_idx.unwrap());
    }

    #[test]
    fn test_add_remove_timed_trigger() {
        let mut ah = AutoHands::new(None);
        ah.add_timed_trigger(TimedTrigger::new("custom-every-10s", 10, "health_check"));
        assert_eq!(ah.timed_triggers.len(), 5);
        ah.remove_timed_trigger("custom-every-10s");
        assert_eq!(ah.timed_triggers.len(), 4);
    }

    #[test]
    fn test_event_trigger_publishes_to_bus() {
        let _guard = event_lock().lock().unwrap();
        reset();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("auto_hands.event_triggered", handler);
        let mut ah = AutoHands::new(Some(bus));
        let event = Event::new("agent.created", "test", serde_json::json!({}));
        ah.handle_event(&event);
        assert_eq!(EVENT_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_maintenance_task_label() {
        assert_eq!(MaintenanceTask::Inspection.label(), "inspection");
        assert_eq!(MaintenanceTask::GarbageCollection.label(), "gc");
        assert_eq!(
            MaintenanceTask::MemoryCompaction.label(),
            "memory_compaction"
        );
        assert_eq!(MaintenanceTask::HealthCheck.label(), "health_check");
        assert_eq!(MaintenanceTask::CacheCleanup.label(), "cache_cleanup");
    }

    #[test]
    fn test_execution_count_tracks_calls() {
        let mut ah = AutoHands::new(None);
        assert_eq!(ah.execution_count, 0);
        ah.enqueue(MaintenanceTask::HealthCheck);
        ah.run();
        assert!(ah.execution_count > 0);
    }
}
