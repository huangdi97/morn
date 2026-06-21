use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::storage::Storage;
use std::path::PathBuf;

pub struct BackupPlugin {
    storage: Option<Storage>,
}

impl MornPlugin for BackupPlugin {
    fn id(&self) -> &str { "morn:backup" }
    fn deps(&self) -> Vec<&str> { vec!["morn:data-layer"] }
    fn priority(&self) -> i32 { 90 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        self.storage = ctx.get::<Storage>("morn:storage");
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
}

impl Default for BackupPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupPlugin {
    pub fn new() -> Self { Self { storage: None } }

    pub fn create_backup(&self, target_path: PathBuf) -> Result<(), PluginError> {
        let storage = self.storage.as_ref().ok_or(PluginError::LoadFailed("morn:backup".into(), "Storage not initialized".into()))?;
        storage.backup_to(target_path).map_err(|e| PluginError::Other(e.to_string()))
    }

    pub fn restore_from(&self, source_path: PathBuf) -> Result<(), PluginError> {
        let storage = self.storage.as_ref().ok_or(PluginError::LoadFailed("morn:backup".into(), "Storage not initialized".into()))?;
        storage.restore_from(source_path).map_err(|e| PluginError::Other(e.to_string()))
    }
}
