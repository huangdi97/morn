use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct VoicePlugin;

impl MornPlugin for VoicePlugin {
    fn id(&self) -> &str {
        "morn:voice"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer"]
    }

    fn priority(&self) -> i32 {
        170
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
}
