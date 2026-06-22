use crate::core::mcp::MCPClient;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use std::sync::{Arc, Mutex};

pub struct McpPlugin(pub Option<MCPClient>);

impl Default for McpPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl McpPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for McpPlugin {
    fn id(&self) -> &str {
        "morn:mcp"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }
    fn priority(&self) -> i32 {
        155
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let storage = ctx.get::<Storage>("morn:storage").ok_or_else(|| {
            PluginError::LoadFailed("morn:mcp".into(), "morn:storage not registered".into())
        })?;
        let event_bus = crate::core::event_bus::SimpleEventBus::new();
        let registry = Arc::new(Mutex::new(Registry::new(Some(storage), Some(event_bus))));
        let client = MCPClient::new(registry);
        ctx.register("morn:mcp", client.clone());
        self.0 = Some(client);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<MCPClient>("morn:mcp").ok_or_else(|| {
            PluginError::ActivateFailed("morn:mcp".into(), "morn:mcp not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
