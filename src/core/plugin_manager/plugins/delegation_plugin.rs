use crate::core::delegation::DelegationManager;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use std::sync::Arc;

pub struct DelegationPlugin(pub Option<Arc<DelegationManager>>);

impl Default for DelegationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DelegationPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for DelegationPlugin {
    fn id(&self) -> &str {
        "morn:delegation"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:orchestrator"]
    }
    fn priority(&self) -> i32 {
        105
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let registry = ctx.get::<Registry>("morn:type-registry").ok_or_else(|| {
            PluginError::LoadFailed(
                "morn:delegation".into(),
                "morn:type-registry not ready".into(),
            )
        })?;
        let manager = Arc::new(DelegationManager::new(Arc::new(registry)));
        ctx.register("morn:delegation", manager.clone());
        self.0 = Some(manager);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<DelegationManager>>("morn:delegation")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:delegation".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
