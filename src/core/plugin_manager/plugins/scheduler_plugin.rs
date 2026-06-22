use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::scheduler::Scheduler;
use std::sync::Arc;

pub struct SchedulerPlugin(pub Option<Arc<Scheduler>>);

impl Default for SchedulerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for SchedulerPlugin {
    fn id(&self) -> &str {
        "morn:scheduler"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:task-engine", "morn:data-layer"]
    }
    fn priority(&self) -> i32 {
        115
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let scheduler = Arc::new(Scheduler::new());
        ctx.register("morn:scheduler", scheduler.clone());
        self.0 = Some(scheduler);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<Scheduler>>("morn:scheduler").ok_or_else(|| {
            PluginError::ActivateFailed("morn:scheduler".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
