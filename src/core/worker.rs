//! worker — Runs background worker tasks and dispatches queued work.
use crate::core::error::MornError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WorkerPool {
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
    running: Arc<Mutex<bool>>,
}

#[allow(dead_code)] /* 预留：工作线程句柄生命周期管理 */
struct WorkerHandle {
    channel_id: String,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new() -> Self {
        WorkerPool {
            workers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), MornError> {
        let mut running = self.running.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        *running = true;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), MornError> {
        let mut running = self.running.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        *running = false;
        let mut workers = self.workers.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        workers.clear();
        Ok(())
    }

    pub fn assign(&self, channel_id: &str) -> Result<(), MornError> {
        let mut workers = self.workers.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        workers.insert(
            channel_id.to_string(),
            WorkerHandle {
                channel_id: channel_id.to_string(),
                thread: None,
            },
        );
        Ok(())
    }

    pub fn remove(&self, channel_id: &str) -> Result<(), MornError> {
        let mut workers = self.workers.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        workers.remove(channel_id);
        Ok(())
    }

    pub fn worker_count(&self) -> usize {
        self.workers.lock().map(|w| w.len()).unwrap_or(0)
    }

    pub fn is_running(&self) -> bool {
        self.running.lock().map(|r| *r).unwrap_or(false)
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CpuWorker {
    id: String,
    pool: WorkerPool,
}

impl CpuWorker {
    pub fn new(id: &str, pool: WorkerPool) -> Self {
        CpuWorker {
            id: id.to_string(),
            pool,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if !self.pool.is_running() {
            return;
        }
        std::thread::spawn(task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn worker_pool_lifecycle_tracks_assigned_workers() {
        let mut pool = WorkerPool::new();

        assert!(!pool.is_running());
        assert_eq!(pool.worker_count(), 0);

        pool.start().unwrap();
        assert!(pool.is_running());

        pool.assign("channel-a").unwrap();
        pool.assign("channel-b").unwrap();
        assert_eq!(pool.worker_count(), 2);

        pool.remove("channel-a").unwrap();
        assert_eq!(pool.worker_count(), 1);

        pool.shutdown().unwrap();
        assert!(!pool.is_running());
        assert_eq!(pool.worker_count(), 0);
    }

    #[test]
    fn cpu_worker_executes_only_when_pool_is_running() {
        let stopped_worker = CpuWorker::new("stopped", WorkerPool::new());
        let (stopped_tx, stopped_rx) = mpsc::channel();

        stopped_worker.execute(move || {
            stopped_tx.send(()).unwrap();
        });

        assert_eq!(stopped_worker.id(), "stopped");
        assert!(stopped_rx.recv_timeout(Duration::from_millis(50)).is_err());

        let mut pool = WorkerPool::new();
        pool.start().unwrap();
        let running_worker = CpuWorker::new("running", pool);
        let (running_tx, running_rx) = mpsc::channel();

        running_worker.execute(move || {
            running_tx.send("done").unwrap();
        });

        assert_eq!(running_worker.id(), "running");
        assert_eq!(
            running_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            "done"
        );
    }
}
