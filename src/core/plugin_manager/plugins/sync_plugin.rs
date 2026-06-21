use crate::bridge::sync::SyncEngine;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct SyncPlugin(pub Option<Arc<Mutex<SyncEngine>>>);

impl Default for SyncPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for SyncPlugin {
    fn id(&self) -> &str {
        "morn:sync"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }

    fn priority(&self) -> i32 {
        130
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed("morn:sync".into(), "morn:storage not found".into())
        })?;

        let server_url = storage
            .get_setting("sync_server_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let device_id = storage
            .get_setting("sync_device_id")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                let id = Uuid::new_v4().to_string();
                let _ = storage.set_setting("sync_device_id", &id);
                id
            });

        let engine = SyncEngine::new(&device_id, Some(server_url))
            .with_storage(Arc::new(Mutex::new(storage.clone())));
        let shared = Arc::new(Mutex::new(engine));
        ctx.register("morn:sync-engine", shared.clone());
        self.0 = Some(shared);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<Mutex<SyncEngine>>>("morn:sync-engine")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:sync".into(), "engine missing".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
