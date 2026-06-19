use crate::core::hub_seeder::seed_hub_data;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;

pub struct DataLayerPlugin(pub Option<Storage>);

impl DataLayerPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for DataLayerPlugin {
    fn id(&self) -> &str {
        "morn:data-layer"
    }

    fn deps(&self) -> Vec<&str> {
        vec![]
    }

    fn priority(&self) -> i32 {
        200
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = Storage::new()
            .map_err(|e| PluginError::LoadFailed("morn:data-layer".to_string(), e.to_string()))?;
        ctx.register("morn:storage", storage.clone());
        self.0 = Some(storage);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:data-layer".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let seeded = storage.get_setting("morn_seeded").ok().flatten();
        if seeded.is_none() {
            seed_hub_data(&Some(storage.clone()));
            storage.set_setting("morn_seeded", "true").ok();
        }
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
