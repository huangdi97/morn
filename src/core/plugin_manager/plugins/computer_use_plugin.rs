use crate::core::computer_control::ComputerControl;
use crate::core::visual_agent::VisualAgent;
use crate::core::visual_grounding::VisualGrounding;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct ComputerUsePlugin;

impl MornPlugin for ComputerUsePlugin {
    fn id(&self) -> &str { "morn:computer-use" }
    fn deps(&self) -> Vec<&str> { vec!["morn:sandbox"] }
    fn priority(&self) -> i32 { 120 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.register("morn:computer-control", ComputerControl);
        ctx.register("morn:visual-agent", VisualAgent::new());
        ctx.register("morn:visual-grounding", VisualGrounding::new());
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
}
