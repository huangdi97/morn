use std::sync::{Arc, Mutex};

use crate::core::component_type::TypeRegistry;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use crate::studio::manager::StudioManager;
use crate::studio::publisher::StudioPublisher;
use crate::studio::tester::StudioTester;

pub struct StudioPlugin(
    pub Option<Arc<Mutex<StudioManager>>>,
    pub Option<Arc<Mutex<StudioPublisher>>>,
);

impl StudioPlugin {
    pub fn new() -> Self {
        Self(None, None)
    }
}

impl MornPlugin for StudioPlugin {
    fn id(&self) -> &str {
        "morn:studio"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer", "morn:registry", "morn:supervisor"]
    }

    fn priority(&self) -> i32 {
        140
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed(
                "morn:studio".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let type_registry = ctx
            .get::<TypeRegistry>("morn:type-registry")
            .ok_or_else(|| {
                PluginError::LoadFailed(
                    "morn:studio".to_string(),
                    "morn:type-registry not registered".to_string(),
                )
            })?;
        let registry = Registry::new(Some(storage.clone()), None);

        let manager = Arc::new(Mutex::new(StudioManager::new(
            Some(registry.clone()),
            Some(storage.clone()),
            None,
        )));
        let publisher = Arc::new(Mutex::new(StudioPublisher::new(
            Some(registry),
            Some(storage),
            Some(type_registry),
        )));

        ctx.register("morn:studio-manager", manager.clone());
        ctx.register("morn:studio-publisher", publisher.clone());

        self.0 = Some(manager);
        self.1 = Some(publisher);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let manager = ctx.get::<Arc<Mutex<StudioManager>>>("morn:studio-manager")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:studio".to_string(),
                    "morn:studio-manager not registered".to_string(),
                )
            })?;
        ctx.get::<Arc<Mutex<StudioPublisher>>>("morn:studio-publisher")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:studio".to_string(),
                    "morn:studio-publisher not registered".to_string(),
                )
            })?;
        ctx.register("morn:studio-manager", manager.clone());
        let publisher = ctx.get::<Arc<Mutex<StudioPublisher>>>("morn:studio-publisher")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:studio".to_string(),
                    "morn:studio-publisher not registered".to_string(),
                )
            })?;
        ctx.register("morn:studio-publisher", publisher.clone());
        let tester = Arc::new(Mutex::new(StudioTester::new()));
        ctx.register("morn:studio-tester", tester);
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}
