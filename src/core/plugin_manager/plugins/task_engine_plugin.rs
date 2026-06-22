use crate::core::long_task_engine::LongTaskEngine;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::Arc;

pub struct TaskEnginePlugin(pub Option<Arc<LongTaskEngine>>);

impl Default for TaskEnginePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskEnginePlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for TaskEnginePlugin {
    fn id(&self) -> &str {
        "morn:task-engine"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer", "morn:sandbox"]
    }
    fn priority(&self) -> i32 {
        145
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let engine = Arc::new(LongTaskEngine::new("init", "init", 1));
        ctx.register("morn:task-engine", engine.clone());
        self.0 = Some(engine);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<LongTaskEngine>>("morn:task-engine")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:task-engine".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
