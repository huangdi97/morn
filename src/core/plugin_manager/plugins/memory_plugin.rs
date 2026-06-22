use crate::core::memory::MemoryHub;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::{Arc, Mutex};

pub struct MemoryPlugin(pub Option<Arc<Mutex<MemoryHub>>>);

impl Default for MemoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for MemoryPlugin {
    fn id(&self) -> &str {
        "morn:memory"
    }
    fn deps(&self) -> Vec<&str> {
        vec![]
    }
    fn priority(&self) -> i32 {
        165
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let hub = Arc::new(Mutex::new(MemoryHub::new()));
        ctx.register("morn:memory-hub", hub.clone());
        self.0 = Some(hub);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<Mutex<MemoryHub>>>("morn:memory-hub")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:memory".into(),
                    "morn:memory-hub not registered".into(),
                )
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
