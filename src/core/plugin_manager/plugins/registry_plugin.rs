use crate::core::component_type::registry::TypeRegistry;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;

pub struct RegistryPlugin(pub Option<TypeRegistry>);

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
        let registry = TypeRegistry::new();
        ctx.register("morn:type-registry", registry.clone());
        self.0 = Some(registry);
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
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:registry".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let registry = Registry::new(Some(storage.clone()), None);
        ctx.register("morn:registry", registry);
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
