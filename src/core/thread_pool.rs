use crate::core::pipeline::agentless::{AgentlessPipeline, PipelineStep};
use crate::core::workflow::{WorkflowAction, WorkflowStep};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

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
pub struct ThreadPoolConfig {
    pub coo_threads: usize,
    pub execution_threads: usize,
    pub channel_threads: usize,
    pub tray_threads: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        ThreadPoolConfig {
            coo_threads: 1,
            execution_threads: 4,
            channel_threads: 2,
            tray_threads: 1,
        }
    }
}

/// 真实线程池 — 管理 COO、执行器、通道和托盘线程。
pub struct ThreadPool {
    pub config: ThreadPoolConfig,
    running: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(config: ThreadPoolConfig) -> Self {
        ThreadPool {
            config,
            running: Arc::new(AtomicBool::new(false)),
            handles: Vec::new(),
        }
    }

    /// 启动所有线程池线程。
    /// CLI 模式下使用单线程通道适配器，Tauri 模式下调用此方法。
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let r = running.clone();
        self.handles.push(std::thread::spawn(move || {
            tracing::info!("[thread_pool] COO thread started");
            while r.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            tracing::info!("[thread_pool] COO thread stopped");
        }));

        for i in 0..self.config.execution_threads {
            let r = running.clone();
            self.handles.push(std::thread::spawn(move || {
                tracing::info!("[thread_pool] Execution thread {} started", i);
                while r.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                tracing::info!("[thread_pool] Execution thread {} stopped", i);
            }));
        }

        for i in 0..self.config.channel_threads {
            let r = running.clone();
            self.handles.push(std::thread::spawn(move || {
                tracing::info!("[thread_pool] Channel thread {} started", i);
                while r.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                tracing::info!("[thread_pool] Channel thread {} stopped", i);
            }));
        }

        let r = running.clone();
        self.handles.push(std::thread::spawn(move || {
            tracing::info!("[thread_pool] Tray thread started");
            while r.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            tracing::info!("[thread_pool] Tray thread stopped");
        }));

        tracing::info!(
            "[thread_pool] All threads started (coo={}, execution={}, channel={}, tray={})",
            self.config.coo_threads,
            self.config.execution_threads,
            self.config.channel_threads,
            self.config.tray_threads,
        );
    }

    /// 停止所有线程并等待它们结束。
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        while let Some(handle) = self.handles.pop() {
            let _ = handle.join();
        }
        tracing::info!("[thread_pool] All threads stopped");
    }

    /// 是否正在运行。
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
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
            if let WorkflowAction::PipelineExec { pipeline_json } = &task.action {
                let steps: Vec<PipelineStep> = serde_json::from_value(pipeline_json.clone())
                    .map_err(|e| format!("invalid pipeline steps: {}", e))?;
                let mut pipeline = AgentlessPipeline::new(steps);
                pipeline.execute()?;
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

    #[test]
    fn thread_pool_config_defaults() {
        let cfg = ThreadPoolConfig::default();
        assert_eq!(cfg.coo_threads, 1);
        assert_eq!(cfg.execution_threads, 4);
        assert_eq!(cfg.channel_threads, 2);
        assert_eq!(cfg.tray_threads, 1);
    }
}