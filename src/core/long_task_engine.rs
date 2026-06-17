//! long_task_engine — Runs long-lived tasks with progress tracking and resumable state.
use crate::core::error::MornError;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub session_id: String,
    pub total_steps: u64,
    pub completed_steps: u64,
    pub current_step: String,
    pub status: TaskStatus,
    pub progress_pct: f64,
    pub checkpoint_data: Value,
    pub errors: Vec<String>,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct LongTaskEngine {
    pub task: TaskProgress,
    heartbeat_interval: Duration,
    last_heartbeat: Arc<AtomicI64>,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

impl LongTaskEngine {
    pub fn new(task_id: &str, session_id: &str, total_steps: u64) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        LongTaskEngine {
            task: TaskProgress {
                task_id: task_id.to_string(),
                session_id: session_id.to_string(),
                total_steps,
                completed_steps: 0,
                current_step: String::new(),
                status: TaskStatus::Pending,
                progress_pct: 0.0,
                checkpoint_data: Value::Null,
                errors: Vec::new(),
                started_at: now.clone(),
                updated_at: now,
            },
            heartbeat_interval: Duration::from_secs(30),
            last_heartbeat: Arc::new(AtomicI64::new(chrono::Utc::now().timestamp())),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn save_progress(&self) -> Result<String, MornError> {
        serde_json::to_string_pretty(&self.task).map_err(|e| MornError::Internal(e.to_string()))
    }

    pub fn load_progress(json: &str) -> Result<Self, MornError> {
        let task: TaskProgress =
            serde_json::from_str(json).map_err(|e| MornError::Internal(e.to_string()))?;
        let now = chrono::Utc::now().timestamp();
        Ok(LongTaskEngine {
            task,
            heartbeat_interval: Duration::from_secs(30),
            last_heartbeat: Arc::new(AtomicI64::new(now)),
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn resume_from_checkpoint(&mut self, json: &str) -> Result<(), MornError> {
        let saved: TaskProgress =
            serde_json::from_str(json).map_err(|e| MornError::Internal(e.to_string()))?;
        self.task.completed_steps = saved.completed_steps;
        self.task.current_step = saved.current_step;
        self.task.status = TaskStatus::Running;
        self.task.checkpoint_data = saved.checkpoint_data;
        self.task.progress_pct = if self.task.total_steps > 0 {
            (self.task.completed_steps as f64 / self.task.total_steps as f64) * 100.0
        } else {
            0.0
        };
        self.task.updated_at = chrono::Utc::now().to_rfc3339();
        self.last_heartbeat
            .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
        Ok(())
    }

    pub fn heartbeat(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        let last = self.last_heartbeat.load(Ordering::Relaxed);
        if now - last >= self.heartbeat_interval.as_secs() as i64 {
            self.last_heartbeat.store(now, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, step_name: &str) {
        if self.cancelled.load(Ordering::Relaxed) {
            self.task.status = TaskStatus::Failed("cancelled".to_string());
        }
        if self.paused.load(Ordering::Relaxed) {
            self.task.status = TaskStatus::Paused;
            return;
        }
        self.task.completed_steps += 1;
        self.task.current_step = step_name.to_string();
        self.task.status = TaskStatus::Running;
        self.task.progress_pct = if self.task.total_steps > 0 {
            (self.task.completed_steps as f64 / self.task.total_steps as f64) * 100.0
        } else {
            0.0
        };
        self.task.updated_at = chrono::Utc::now().to_rfc3339();
        self.last_heartbeat
            .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn record_error(&mut self, error: &str) {
        self.task.errors.push(error.to_string());
        self.task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn complete(&mut self) {
        self.task.completed_steps = self.task.total_steps;
        self.task.progress_pct = 100.0;
        self.task.status = TaskStatus::Completed;
        self.task.updated_at = chrono::Utc::now().to_rfc3339();
        self.last_heartbeat
            .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn set_checkpoint(&mut self, data: Value) {
        self.task.checkpoint_data = data;
        self.task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn elapsed(&self) -> Duration {
        let start = chrono::DateTime::parse_from_rfc3339(&self.task.started_at)
            .unwrap_or_else(|_| chrono::DateTime::from(chrono::Utc::now()));
        let elapsed = chrono::Utc::now() - start.with_timezone(&chrono::Utc);
        elapsed.to_std().unwrap_or(Duration::ZERO)
    }

    pub fn estimated_remaining(&self) -> Option<Duration> {
        if self.task.completed_steps == 0 || self.task.progress_pct <= 0.0 {
            return None;
        }
        let elapsed = self.elapsed();
        let rate = elapsed.as_secs_f64() / self.task.completed_steps as f64;
        let remaining = (self.task.total_steps - self.task.completed_steps) as f64 * rate;
        Some(Duration::from_secs_f64(remaining))
    }

    pub fn set_heartbeat_interval(&mut self, secs: u64) {
        self.heartbeat_interval = Duration::from_secs(secs);
    }

    pub fn last_heartbeat_at(&self) -> i64 {
        self.last_heartbeat.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine_starts_pending_with_empty_progress() {
        let engine = LongTaskEngine::new("task-1", "session-1", 4);

        assert_eq!(engine.task.task_id, "task-1");
        assert_eq!(engine.task.session_id, "session-1");
        assert_eq!(engine.task.total_steps, 4);
        assert_eq!(engine.task.completed_steps, 0);
        assert_eq!(engine.task.progress_pct, 0.0);
        assert!(matches!(engine.task.status, TaskStatus::Pending));
        assert!(!engine.is_paused());
        assert!(!engine.is_cancelled());
    }

    #[test]
    fn tick_pause_resume_and_complete_update_task_state() {
        let mut engine = LongTaskEngine::new("task-1", "session-1", 4);

        engine.tick("load");
        assert_eq!(engine.task.completed_steps, 1);
        assert_eq!(engine.task.current_step, "load");
        assert_eq!(engine.task.progress_pct, 25.0);
        assert!(matches!(engine.task.status, TaskStatus::Running));
        assert!(engine.estimated_remaining().is_some());

        engine.pause();
        engine.tick("paused-step");
        assert!(engine.is_paused());
        assert_eq!(engine.task.completed_steps, 1);
        assert!(matches!(engine.task.status, TaskStatus::Paused));

        engine.resume();
        engine.tick("process");
        assert!(!engine.is_paused());
        assert_eq!(engine.task.completed_steps, 2);
        assert_eq!(engine.task.progress_pct, 50.0);

        engine.complete();
        assert_eq!(engine.task.completed_steps, 4);
        assert_eq!(engine.task.progress_pct, 100.0);
        assert!(matches!(engine.task.status, TaskStatus::Completed));
    }

    #[test]
    fn progress_serializes_loads_and_resumes_from_checkpoint() {
        let mut engine = LongTaskEngine::new("task-1", "session-1", 4);
        engine.set_checkpoint(serde_json::json!({"cursor": 2}));
        engine.tick("read");
        engine.record_error("transient");

        let saved = engine.save_progress().unwrap();
        let loaded = LongTaskEngine::load_progress(&saved).unwrap();

        assert_eq!(loaded.task.task_id, "task-1");
        assert_eq!(loaded.task.completed_steps, 1);
        assert_eq!(
            loaded.task.checkpoint_data,
            serde_json::json!({"cursor": 2})
        );
        assert_eq!(loaded.task.errors, vec!["transient"]);

        let mut resumed = LongTaskEngine::new("task-2", "session-2", 4);
        resumed.resume_from_checkpoint(&saved).unwrap();

        assert_eq!(resumed.task.completed_steps, 1);
        assert_eq!(resumed.task.current_step, "read");
        assert_eq!(resumed.task.progress_pct, 25.0);
        assert_eq!(
            resumed.task.checkpoint_data,
            serde_json::json!({"cursor": 2})
        );
        assert!(matches!(resumed.task.status, TaskStatus::Running));
    }

    #[test]
    fn heartbeat_and_cancel_flags_are_observable() {
        let mut engine = LongTaskEngine::new("task-1", "session-1", 1);

        engine.set_heartbeat_interval(0);
        assert!(engine.heartbeat());
        assert!(engine.last_heartbeat_at() > 0);

        engine.cancel();
        assert!(engine.is_cancelled());
    }
}
