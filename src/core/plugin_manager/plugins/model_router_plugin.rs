use crate::core::model_router::ModelRouter;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct ModelRouterPlugin(pub Option<ModelRouter>);

impl Default for ModelRouterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouterPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for ModelRouterPlugin {
    fn id(&self) -> &str {
        "morn:model-router"
    }
    fn deps(&self) -> Vec<&str> {
        vec![]
    }
    fn priority(&self) -> i32 {
        170
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let router = ModelRouter::default();
        ctx.register("morn:model-router", router.clone());
        self.0 = Some(router);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<ModelRouter>("morn:model-router").ok_or_else(|| {
            PluginError::ActivateFailed("morn:model-router".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
