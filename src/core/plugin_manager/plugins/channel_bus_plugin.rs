use std::sync::{Arc, Mutex};

use crate::channel::adapter::ChannelAdapter;
use crate::console::ConsoleBackend;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;

pub struct ChannelBusPlugin(pub Option<Arc<Mutex<ChannelAdapter>>>);

impl Default for ChannelBusPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelBusPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for ChannelBusPlugin {
    fn id(&self) -> &str {
        "morn:channel-bus"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }

    fn priority(&self) -> i32 {
        180
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let adapter = ChannelAdapter::new(None);
        let shared = Arc::new(Mutex::new(adapter));
        ctx.register("morn:channel-adapter", shared.clone());
        self.0 = Some(shared);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:channel-bus".to_string(),
                "morn:storage not registered".to_string(),
            )
        })?;
        let registry = ctx.get::<Registry>("morn:registry").ok_or_else(|| {
            PluginError::ActivateFailed(
                "morn:channel-bus".to_string(),
                "morn:registry not registered".to_string(),
            )
        })?;
        let console = ConsoleBackend::new(Some(registry), Some(storage), None, None, None, None);
        ctx.register("morn:console", Arc::new(Mutex::new(console)));

        ctx.get::<Arc<Mutex<ChannelAdapter>>>("morn:channel-adapter")
            .ok_or_else(|| {
                PluginError::ActivateFailed(
                    "morn:channel-bus".to_string(),
                    "morn:channel-adapter not registered".to_string(),
                )
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
