use std::sync::Arc;
use crate::core::assembly::AssemblyBuilder;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct AssemblyPlugin(pub Option<Arc<AssemblyBuilder>>);

impl Default for AssemblyPlugin {
    fn default() -> Self { Self::new() }
}

impl AssemblyPlugin {
    pub fn new() -> Self { Self(None) }
}

impl MornPlugin for AssemblyPlugin {
    fn id(&self) -> &str { "morn:assembly" }
    fn deps(&self) -> Vec<&str> { vec!["morn:registry", "morn:component-type"] }
    fn priority(&self) -> i32 { 80 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let builder = Arc::new(AssemblyBuilder::new());
        ctx.register("morn:assembly", builder.clone());
        self.0 = Some(builder);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<AssemblyBuilder>>("morn:assembly").ok_or_else(|| {
            PluginError::ActivateFailed("morn:assembly".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}