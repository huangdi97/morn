use std::sync::Arc;
use crate::core::proactive::ProactiveEngine;
use crate::core::storage::Storage;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct ProactivePlugin(pub Option<Arc<ProactiveEngine>>);

impl Default for ProactivePlugin {
    fn default() -> Self { Self::new() }
}

impl ProactivePlugin {
    pub fn new() -> Self { Self(None) }
}

impl MornPlugin for ProactivePlugin {
    fn id(&self) -> &str { "morn:proactive" }
    fn deps(&self) -> Vec<&str> { vec!["morn:scheduler", "morn:engine"] }
    fn priority(&self) -> i32 { 110 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage")
            .ok_or_else(|| PluginError::LoadFailed("morn:proactive".into(), "morn:storage not ready".into()))?;
        let engine = Arc::new(ProactiveEngine::new(Some(Arc::new(storage))));
        ctx.register("morn:proactive", engine.clone());
        self.0 = Some(engine);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<ProactiveEngine>>("morn:proactive").ok_or_else(|| {
            PluginError::ActivateFailed("morn:proactive".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}