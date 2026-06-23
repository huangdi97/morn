use std::sync::Arc;

use crate::core::component_type::registry::TypeRegistry;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;

pub struct RegistryPlugin(pub Option<TypeRegistry>);

impl Default for RegistryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for RegistryPlugin {
    fn id(&self) -> &str {
        "morn:registry"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }

    fn priority(&self) -> i32 {
        190
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed("morn:registry".into(), "morn:storage not ready".into())
        })?;
        let type_registry = TypeRegistry::new();
        ctx.register("morn:type-registry", type_registry.clone());
        let registry = Arc::new(Registry::new(Some(storage.clone()), None));
        ctx.register("morn:registry", registry);
        self.0 = Some(type_registry);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<TypeRegistry>("morn:type-registry")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:registry".to_string(),
                    "morn:type-registry not registered".to_string(),
                )
            })?;
        ctx.get::<Arc<Registry>>("morn:registry").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:registry".to_string(),
                "morn:registry not registered".to_string(),
            )
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
