use crate::core::agent_pool::{AgentPool, PoolConfig};
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::Arc;

pub struct AgentPoolPlugin(pub Option<Arc<AgentPool>>);

impl Default for AgentPoolPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentPoolPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for AgentPoolPlugin {
    fn id(&self) -> &str {
        "morn:agent-pool"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:orchestrator", "morn:memory", "morn:task-engine"]
    }
    fn priority(&self) -> i32 {
        95
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let pool = Arc::new(AgentPool::new(PoolConfig::default()));
        ctx.register("morn:agent-pool", pool.clone());
        self.0 = Some(pool);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<AgentPool>>("morn:agent-pool")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:agent-pool".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
