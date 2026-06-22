use std::sync::Arc;
use crate::core::approval::ApprovalManager;
use crate::core::storage::Storage;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct ApprovalPlugin(pub Option<Arc<ApprovalManager>>);

impl Default for ApprovalPlugin {
    fn default() -> Self { Self::new() }
}

impl ApprovalPlugin {
    pub fn new() -> Self { Self(None) }
}

impl MornPlugin for ApprovalPlugin {
    fn id(&self) -> &str { "morn:approval" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer", "morn:security"] }
    fn priority(&self) -> i32 { 140 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage")
            .ok_or_else(|| PluginError::LoadFailed("morn:approval".into(), "morn:storage not ready".into()))?;
        let manager = Arc::new(ApprovalManager::new(Arc::new(storage), None));
        ctx.register("morn:approval", manager.clone());
        self.0 = Some(manager);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<ApprovalManager>>("morn:approval").ok_or_else(|| {
            PluginError::ActivateFailed("morn:approval".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
