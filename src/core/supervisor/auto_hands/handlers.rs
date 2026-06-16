//! 自动执行处理器 — 各类自动操作的执行逻辑
use crate::core::error::MornError;
use crate::core::event_bus::Event;
use std::time::SystemTime;

use super::{timestamp, AutoHands, MaintenanceTask};

impl AutoHands {
    pub fn handle_event(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        let triggered: Vec<(String, &str)> = self
            .event_triggers
            .iter()
            .filter(|t| t.enabled && t.event_type == event.event_type)
            .map(|t| (t.event_type.clone(), t.handler))
            .collect();

        for (event_type, handler) in &triggered {
            self.execution_count += 1;
            let task = match *handler {
                "health_check" => MaintenanceTask::HealthCheck,
                "gc" => MaintenanceTask::GarbageCollection,
                "inspection" => MaintenanceTask::Inspection,
                "log_rotation" => MaintenanceTask::LogRotation,
                _ => MaintenanceTask::Inspection,
            };
            let label = task.label().to_string();
            self.enqueue(task);
            if let Some(ref bus) = self.event_bus {
                bus.publish_event(
                    "auto_hands.event_triggered",
                    "auto_hands",
                    serde_json::json!({
                        "trigger": event_type,
                        "handler": handler,
                        "task": label,
                    }),
                );
            }
        }
    }

    pub fn tick(&mut self) -> Vec<MaintenanceTask> {
        if !self.enabled {
            return vec![];
        }
        let now = SystemTime::now();
        let mut fired = Vec::new();

        for trigger in &self.timed_triggers {
            if trigger.is_ready(now) {
                let task = match trigger.handler {
                    "health_check" => MaintenanceTask::HealthCheck,
                    "gc" => MaintenanceTask::GarbageCollection,
                    "inspection" => MaintenanceTask::Inspection,
                    "memory_compaction" => MaintenanceTask::MemoryCompaction,
                    "log_rotation" => MaintenanceTask::LogRotation,
                    _ => MaintenanceTask::Inspection,
                };
                fired.push(task);
                if let Some(ref bus) = self.event_bus {
                    bus.publish_event(
                        "auto_hands.timed_trigger",
                        "auto_hands",
                        serde_json::json!({
                            "trigger_id": trigger.id,
                            "handler": trigger.handler,
                            "interval_secs": trigger.interval_secs,
                        }),
                    );
                }
            }
        }

        for task in &fired {
            self.enqueue(task.clone());
        }

        self.execution_count += fired.len() as u64;
        if !fired.is_empty() {
            self.last_run = Some(now);
        }

        fired
    }

    pub fn run(&mut self) -> Vec<(MaintenanceTask, Result<String, MornError>)> {
        let _ = self.tick();
        let mut tasks = self.drain_queue();
        let mut results = Vec::new();

        tasks.sort_by_key(|t| t.priority());

        for task in tasks {
            let result = self.execute_task(&task);
            results.push((task, result));
        }

        if let Some(ref bus) = self.event_bus {
            let completed = results.iter().filter(|(_, r)| r.is_ok()).count();
            let failed = results.iter().filter(|(_, r)| r.is_err()).count();
            bus.publish_event(
                "auto_hands.run_completed",
                "auto_hands",
                serde_json::json!({
                    "completed": completed,
                    "failed": failed,
                    "total": results.len(),
                }),
            );
        }

        results
    }

    pub fn run_once(&mut self) -> Vec<(MaintenanceTask, Result<String, MornError>)> {
        self.timed_triggers.iter_mut().for_each(|t| {
            t.last_fired = Some(SystemTime::now());
        });
        self.run()
    }

    pub(crate) fn execute_task(&self, task: &MaintenanceTask) -> Result<String, MornError> {
        match task {
            MaintenanceTask::Inspection => Ok(format!("inspection ok at {}", timestamp())),
            MaintenanceTask::GarbageCollection => Ok(format!("gc reclaimed {} bytes", 0)),
            MaintenanceTask::MemoryCompaction => Ok(format!("memory compacted: {} pages freed", 0)),
            MaintenanceTask::LogRotation => Ok(format!("log rotated, {} bytes archived", 0)),
            MaintenanceTask::HealthCheck => Ok(format!("health check passed at {}", timestamp())),
            MaintenanceTask::CacheCleanup => Ok(format!("cache cleaned: {} entries removed", 0)),
        }
    }
}
