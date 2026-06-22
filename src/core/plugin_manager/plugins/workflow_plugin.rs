use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use crate::core::registry::Registry;
use crate::core::workflow_builder::WorkflowBuilder;
use std::sync::Arc;

pub struct WorkflowPlugin(pub Option<Arc<WorkflowBuilder>>);

impl Default for WorkflowPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for WorkflowPlugin {
    fn id(&self) -> &str {
        "morn:workflow"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:task-engine", "morn:registry"]
    }
    fn priority(&self) -> i32 {
        135
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let registry = ctx
            .get::<Arc<Registry>>("morn:type-registry")
            .ok_or_else(|| {
                PluginError::LoadFailed(
                    "morn:workflow".into(),
                    "morn:type-registry not ready".into(),
                )
            })?;
        let builder = Arc::new(WorkflowBuilder::new(registry));
        ctx.register("morn:workflow", builder.clone());
        self.0 = Some(builder);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<WorkflowBuilder>>("morn:workflow")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:workflow".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
