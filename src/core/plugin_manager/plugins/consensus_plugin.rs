use crate::core::consensus::ConsensusManager;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::path::Path;
use std::sync::Arc;

pub struct ConsensusPlugin(pub Option<Arc<ConsensusManager>>);

impl Default for ConsensusPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsensusPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for ConsensusPlugin {
    fn id(&self) -> &str {
        "morn:consensus"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:orchestrator"]
    }
    fn priority(&self) -> i32 {
        100
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let consensus_dir = Path::new("./data/consensus");
        let manager = Arc::new(ConsensusManager::new(consensus_dir));
        ctx.register("morn:consensus", manager.clone());
        self.0 = Some(manager);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<ConsensusManager>>("morn:consensus")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:consensus".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
