use std::sync::{Arc, Mutex};

use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use crate::core::supervisor::presets::seed_preset_agents;
use crate::core::supervisor::Supervisor;
use crate::studio::manager::StudioManager;

pub struct SupervisorPlugin(pub Option<Arc<Mutex<Supervisor>>>);

impl SupervisorPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for SupervisorPlugin {
    fn id(&self) -> &str {
        "morn:supervisor"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer", "morn:engine"]
    }

    fn priority(&self) -> i32 {
        150
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed(
                "morn:supervisor".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let supervisor = Supervisor::new(Some(storage), None);
        let shared = Arc::new(Mutex::new(supervisor));
        ctx.register("morn:supervisor", shared.clone());
        self.0 = Some(shared);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:supervisor".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let registry = Registry::new(Some(storage.clone()), None);
        let manager = StudioManager::new(Some(registry), Some(storage.clone()), None);
        seed_preset_agents(&Some(storage), &manager);

        ctx.get::<Arc<Mutex<Supervisor>>>("morn:supervisor")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:supervisor".to_string(),
                    "morn:supervisor not registered".to_string(),
                )
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
