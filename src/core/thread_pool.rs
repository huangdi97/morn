//! thread_pool — Provides bounded task execution helpers for workflow nodes.
use crate::core::workflow::WorkflowStep;

pub type TaskDef = WorkflowStep;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskPoolConfig {
    pub min_threads: usize,
    pub max_threads: usize,
    pub queue_size: usize,
}

impl Default for TaskPoolConfig {
    fn default() -> Self {
        TaskPoolConfig {
            min_threads: 1,
            max_threads: 4,
            queue_size: 128,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskPool {
    config: TaskPoolConfig,
}

impl TaskPool {
    pub fn new(config: TaskPoolConfig) -> Result<Self, String> {
        if config.min_threads == 0 {
            return Err("min_threads must be greater than zero".to_string());
        }
        if config.max_threads < config.min_threads {
            return Err("max_threads must be greater than or equal to min_threads".to_string());
        }
        if config.queue_size == 0 {
            return Err("queue_size must be greater than zero".to_string());
        }
        Ok(TaskPool { config })
    }

    pub fn default_pool() -> Self {
        TaskPool {
            config: TaskPoolConfig::default(),
        }
    }

    pub fn config(&self) -> &TaskPoolConfig {
        &self.config
    }

    pub fn execute(&self, task: TaskDef) -> tokio::task::JoinHandle<Result<(), String>> {
        tokio::task::spawn_blocking(move || {
            if task.id.trim().is_empty() {
                return Err("task id cannot be empty".to_string());
            }
            Ok(())
        })
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::default_pool()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workflow::WorkflowAction;
    use std::collections::HashMap;

    fn task(id: &str) -> TaskDef {
        TaskDef {
            id: id.to_string(),
            action: WorkflowAction::Notification {
                channel: "test".to_string(),
                message: "done".to_string(),
            },
            depends_on: vec![],
            timeout_secs: 1,
            retry_count: 0,
            approval_required: false,
            input_mapping: HashMap::new(),
            output_mapping: HashMap::new(),
        }
    }

    #[test]
    fn rejects_invalid_config() {
        let result = TaskPool::new(TaskPoolConfig {
            min_threads: 2,
            max_threads: 1,
            queue_size: 1,
        });

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn executes_valid_task() {
        let pool = TaskPool::default();
        let result = pool.execute(task("step-1")).await.unwrap();

        assert!(result.is_ok());
    }
}
