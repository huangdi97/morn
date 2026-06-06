use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WorkerPool {
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
    running: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
struct WorkerHandle {
    channel_id: String,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerPool {
    pub fn new() -> Self {
        WorkerPool {
            workers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let mut running = self.running.lock().map_err(|e| e.to_string())?;
        *running = true;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        let mut running = self.running.lock().map_err(|e| e.to_string())?;
        *running = false;
        let mut workers = self.workers.lock().map_err(|e| e.to_string())?;
        workers.clear();
        Ok(())
    }

    pub fn assign(&self, channel_id: &str) -> Result<(), String> {
        let mut workers = self.workers.lock().map_err(|e| e.to_string())?;
        workers.insert(
            channel_id.to_string(),
            WorkerHandle {
                channel_id: channel_id.to_string(),
                thread: None,
            },
        );
        Ok(())
    }

    pub fn remove(&self, channel_id: &str) -> Result<(), String> {
        let mut workers = self.workers.lock().map_err(|e| e.to_string())?;
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
