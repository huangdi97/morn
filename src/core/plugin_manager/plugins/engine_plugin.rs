use crate::core::engine::TaskEngine;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;

pub struct EnginePlugin(pub Option<TaskEngine>);

impl EnginePlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for EnginePlugin {
    fn id(&self) -> &str {
        "morn:engine"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer", "morn:registry", "morn:sandbox"]
    }

    fn priority(&self) -> i32 {
        160
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed(
                "morn:engine".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let engine = TaskEngine::new(Some(storage), None);
        ctx.register("morn:task-engine", engine.clone());
        self.0 = Some(engine);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<TaskEngine>("morn:task-engine").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:engine".to_string(),
                "morn:task-engine not registered".to_string(),
            )
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
