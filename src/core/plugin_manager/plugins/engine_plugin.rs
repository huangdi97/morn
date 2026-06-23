use crate::core::engine::TaskEngine;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;

pub struct EnginePlugin(pub Option<TaskEngine>);

impl Default for EnginePlugin {
    fn default() -> Self {
        Self::new()
    }
}

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
            PluginError::LoadFailed("morn:engine".into(), "morn:storage not registered".into())
        })?;
        let engine = TaskEngine::new(Some(storage), None);
        self.0 = Some(engine);
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        if self.0.is_none() {
            return Err(PluginError::ActivateFailed(
                "morn:engine".into(),
                "not initialized".into(),
            ));
        }
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
