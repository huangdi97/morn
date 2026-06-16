use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;
use uuid::Uuid;

pub type TaskHandler = Arc<dyn Fn(ScheduledTask) + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ScheduleType {
    Once { delay_seconds: u64 },
    Interval { interval_seconds: u64 },
    Cron { expression: String },
}

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: String,
    pub agent_id: String,
    pub input: String,
    pub schedule_type: ScheduleType,
    pub next_run_at: i64,
    pub max_runs: Option<u32>,
    pub run_count: u32,
    pub enabled: bool,
}

impl ScheduledTask {
    fn new(
        agent_id: String,
        input: String,
        schedule_type: ScheduleType,
        max_runs: Option<u32>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let delay_seconds = match &schedule_type {
            ScheduleType::Once { delay_seconds } => *delay_seconds,
            ScheduleType::Interval { interval_seconds } => *interval_seconds,
            ScheduleType::Cron { .. } => 0,
        };
        let next_run_at = Utc::now().timestamp() + delay_seconds as i64;
        ScheduledTask {
            id,
            agent_id,
            input,
            schedule_type,
            next_run_at,
            max_runs,
            run_count: 0,
            enabled: true,
        }
    }

    fn should_run(&self, now: i64) -> bool {
        self.enabled && self.next_run_at <= now
    }

    fn reschedule(&mut self) {
        match &self.schedule_type {
            ScheduleType::Once { .. } => {
                self.enabled = false;
            }
            ScheduleType::Interval { interval_seconds } => {
                self.next_run_at = Utc::now().timestamp() + *interval_seconds as i64;
            }
            ScheduleType::Cron { .. } => {
                self.enabled = false;
            }
        }
        self.run_count += 1;
        if let Some(max) = self.max_runs {
            if self.run_count >= max {
                self.enabled = false;
            }
        }
    }
}

pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<String, ScheduledTask>>>,
    handler: Option<TaskHandler>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            handler: None,
            shutdown_tx: None,
        }
    }

    pub fn set_handler(&mut self, handler: TaskHandler) {
        self.handler = Some(handler);
    }

    pub async fn add_task(
        &self,
        agent_id: String,
        input: String,
        schedule_type: ScheduleType,
        max_runs: Option<u32>,
    ) -> String {
        let task = ScheduledTask::new(agent_id, input, schedule_type, max_runs);
        let id = task.id.clone();
        let mut tasks = self.tasks.lock().await;
        tasks.insert(id.clone(), task);
        id
    }

    pub async fn remove_task(&self, id: &str) -> bool {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(id).is_some()
    }

    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.lock().await;
        tasks.values().cloned().collect()
    }

    pub fn start(&mut self) {
        let (tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(tx.clone());

        let tasks = self.tasks.clone();
        let handler = self.handler.clone();

        tokio::spawn(async move {
            let mut rx = tx.subscribe();
            info!("Scheduler started");
            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        info!("Scheduler stopped");
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                        let now = Utc::now().timestamp();
                        let mut due_tasks = Vec::new();
                        {
                            let mut guard = tasks.lock().await;
                            for task in guard.values_mut() {
                                if task.should_run(now) {
                                    due_tasks.push(task.clone());
                                    task.reschedule();
                                }
                            }
                        }
                        for task in due_tasks {
                            info!(id = %task.id, agent_id = %task.agent_id, "Executing scheduled task");
                            if let Some(ref handler) = handler {
                                handler(task);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}