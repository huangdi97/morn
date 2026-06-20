use super::context::PluginContext;
use super::error::PluginError;
use super::types::PluginType;

pub trait TypedPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn version(&self) -> &str;
    fn load(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn deactivate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn unload(&self) -> Result<(), PluginError>;
    fn hooks(&self) -> Vec<&str> {
        vec![]
    }
}

pub trait MornPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn deps(&self) -> Vec<&str> {
        vec![]
    }
    fn priority(&self) -> i32 {
        0
    }
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn deactivate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
}