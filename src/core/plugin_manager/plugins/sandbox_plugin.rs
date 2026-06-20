use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::sandbox::wasm::Sandbox;

pub struct SandboxPlugin(pub Option<Sandbox>);

impl Default for SandboxPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for SandboxPlugin {
    fn id(&self) -> &str {
        "morn:sandbox"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:registry"]
    }

    fn priority(&self) -> i32 {
        170
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let sandbox = Sandbox::new()
            .map_err(|e| PluginError::LoadFailed("morn:sandbox".to_string(), e.to_string()))?;
        ctx.register("morn:sandbox", sandbox.clone());
        self.0 = Some(sandbox);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Sandbox>("morn:sandbox").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:sandbox".to_string(),
                "morn:sandbox not registered".to_string(),
            )
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
