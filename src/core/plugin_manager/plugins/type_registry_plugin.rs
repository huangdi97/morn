use crate::core::component_type::registry::TypeRegistry;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::Arc;

pub struct TypeRegistryPlugin(pub Option<Arc<TypeRegistry>>);

impl Default for TypeRegistryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistryPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for TypeRegistryPlugin {
    fn id(&self) -> &str {
        "morn:component-type"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:registry"]
    }
    fn priority(&self) -> i32 {
        85
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let registry = Arc::new(TypeRegistry::new());
        ctx.register("morn:component-type", registry.clone());
        self.0 = Some(registry);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<TypeRegistry>>("morn:component-type")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:component-type".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
