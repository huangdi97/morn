use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;
use std::sync::Arc;

pub struct ObservabilityPlugin(pub Option<Arc<Storage>>);

impl Default for ObservabilityPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservabilityPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for ObservabilityPlugin {
    fn id(&self) -> &str {
        "morn:observability"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }

    fn priority(&self) -> i32 {
        175
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx
            .get::<Storage>("morn:storage")
            .ok_or_else(|| {
                PluginError::LoadFailed(
                    "morn:observability".into(),
                    "morn:storage not found".into(),
                )
            })?;
        ctx.register("morn:observability", storage.clone());
        self.0 = Some(Arc::new(storage));
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx
            .get::<Storage>("morn:observability")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:observability".into(), "missing".into())
            })?;
        let _ = storage.get_cost_summary(30);
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
