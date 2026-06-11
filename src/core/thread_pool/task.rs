//! task — TaskPool, TaskDef, and pipeline task execution.
use crate::core::pipeline::PipelineTask;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub type TaskDef = PipelineTask;
pub type TaskSender = Sender<PipelineTask>;

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

#[derive(Clone)]
pub struct TaskPool {
    config: TaskPoolConfig,
    sender: TaskSender,
    _receiver: Option<Arc<Mutex<Receiver<PipelineTask>>>>,
}

impl std::fmt::Debug for TaskPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskPool")
            .field("config", &self.config)
            .finish()
    }
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
        let (sender, receiver) = mpsc::channel();
        Ok(TaskPool {
            config,
            sender,
            _receiver: Some(Arc::new(Mutex::new(receiver))),
        })
    }

    pub fn with_sender(config: TaskPoolConfig, sender: TaskSender) -> Result<Self, String> {
        if config.min_threads == 0 {
            return Err("min_threads must be greater than zero".to_string());
        }
        if config.max_threads < config.min_threads {
            return Err("max_threads must be greater than or equal to min_threads".to_string());
        }
        if config.queue_size == 0 {
            return Err("queue_size must be greater than zero".to_string());
        }
        Ok(TaskPool {
            config,
            sender,
            _receiver: None,
        })
    }

    pub fn default_pool() -> Self {
        TaskPool::new(TaskPoolConfig::default())
            .unwrap_or_else(|e| panic!("default task pool config is valid: {}", e))
    }

    pub fn config(&self) -> &TaskPoolConfig {
        &self.config
    }

    pub fn execute(&self, task: TaskDef) -> tokio::task::JoinHandle<Result<(), String>> {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if task.id.trim().is_empty() {
                return Err("task id cannot be empty".to_string());
            }
            sender
                .send(task)
                .map_err(|e| format!("failed to enqueue task: {}", e))
        })
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::default_pool()
    }
}

pub(crate) fn execute_pipeline_task(task: PipelineTask) -> Result<(), String> {
    use crate::core::pipeline::agentless::{AgentlessPipeline, PipelineStep};
    use crate::core::workflow::WorkflowAction;

    if task.id.trim().is_empty() {
        return Err("task id cannot be empty".to_string());
    }
    if let WorkflowAction::PipelineExec { pipeline_json } = &task.action {
        let steps: Vec<PipelineStep> = serde_json::from_value(pipeline_json.clone())
            .map_err(|e| format!("invalid pipeline steps: {}", e))?;
        let mut pipeline = AgentlessPipeline::new(steps);
        pipeline.execute()?;
    }
    Ok(())
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
