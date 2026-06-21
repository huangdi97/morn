use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;

pub struct HubPlugin;

impl Default for HubPlugin {
    fn default() -> Self { Self }
}

impl MornPlugin for HubPlugin {
    fn id(&self) -> &str { "morn:hub" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer", "morn:studio"] }
    fn priority(&self) -> i32 { 130 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let _storage: Storage = ctx.get::<Storage>("morn:storage")
            .ok_or_else(|| PluginError::LoadFailed(
                "morn:hub".into(),
                "Storage not available".into(),
            ))?;
        Ok(())
    }
    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
}
