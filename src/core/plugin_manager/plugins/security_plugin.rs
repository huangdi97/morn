use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::security::SecurityGuard;
use crate::core::PrivacyGate;
use std::sync::Arc;

pub struct SecurityPlugin(pub Option<Arc<SecurityGuard>>);

impl Default for SecurityPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for SecurityPlugin {
    fn id(&self) -> &str {
        "morn:security"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }
    fn priority(&self) -> i32 {
        160
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let guard = Arc::new(SecurityGuard::new());
        ctx.register("morn:security-guard", guard.clone());
        self.0 = Some(guard);

        let gate = Arc::new(PrivacyGate::new());
        ctx.register("morn:privacy-gate", gate);

        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<SecurityGuard>>("morn:security-guard")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:security".into(),
                    "morn:security-guard not registered".into(),
                )
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
