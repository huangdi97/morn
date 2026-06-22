use crate::core::orchestrator::Orchestrator;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::{Arc, Mutex};

pub struct OrchestratorPlugin(pub Option<Arc<Mutex<Orchestrator>>>);

impl Default for OrchestratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestratorPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for OrchestratorPlugin {
    fn id(&self) -> &str {
        "morn:orchestrator"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:task-engine", "morn:registry", "morn:memory"]
    }
    fn priority(&self) -> i32 {
        130
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let orchestrator = Arc::new(Mutex::new(Orchestrator::new(None, None, None)));
        ctx.register("morn:orchestrator", orchestrator.clone());
        self.0 = Some(orchestrator);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<Mutex<Orchestrator>>>("morn:orchestrator")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:orchestrator".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
