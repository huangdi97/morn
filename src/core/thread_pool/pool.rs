//! pool — ThreadPool and ThreadPoolConfig for managing OS threads.
use crate::core::pipeline::PipelineTask;
use crate::core::thread_pool::task::execute_pipeline_task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    task_sender: Sender<PipelineTask>,
    task_receiver: Arc<Mutex<Receiver<PipelineTask>>>,
}

impl ThreadPool {
    pub fn new(config: ThreadPoolConfig) -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        ThreadPool {
            config,
            running: Arc::new(AtomicBool::new(false)),
            handles: Vec::new(),
            task_sender,
            task_receiver: Arc::new(Mutex::new(task_receiver)),
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
            let receiver = self.task_receiver.clone();
            self.handles.push(std::thread::spawn(move || {
                tracing::info!("[thread_pool] Execution thread {} started", i);
                loop {
                    if !r.load(Ordering::Relaxed) {
                        break;
                    }
                    let next_task = match receiver.lock() {
                        Ok(rx) => rx.try_recv(),
                        Err(e) => {
                            tracing::error!(
                                "[thread_pool] Execution thread {} receiver lock failed: {}",
                                i,
                                e
                            );
                            break;
                        }
                    };
                    match next_task {
                        Ok(task) => {
                            if let Err(e) = execute_pipeline_task(task) {
                                tracing::error!(
                                    "[thread_pool] Execution thread {} task failed: {}",
                                    i,
                                    e
                                );
                            }
                        }
                        Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(10)),
                        Err(TryRecvError::Disconnected) => break,
                    }
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

    pub fn task_sender(&self) -> Sender<PipelineTask> {
        self.task_sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_pool_config_defaults() {
        let cfg = ThreadPoolConfig::default();
        assert_eq!(cfg.coo_threads, 1);
        assert_eq!(cfg.execution_threads, 4);
        assert_eq!(cfg.channel_threads, 2);
        assert_eq!(cfg.tray_threads, 1);
    }
}
