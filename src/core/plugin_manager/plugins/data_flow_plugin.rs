use crate::core::data_flow::DataFlowLogger;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::Arc;

pub struct DataFlowPlugin(pub Option<Arc<DataFlowLogger>>);

impl Default for DataFlowPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFlowPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for DataFlowPlugin {
    fn id(&self) -> &str {
        "morn:data-flow"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:task-engine"]
    }
    fn priority(&self) -> i32 {
        125
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let logger = Arc::new(DataFlowLogger::new(1000));
        ctx.register("morn:data-flow", logger.clone());
        self.0 = Some(logger);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<DataFlowLogger>>("morn:data-flow")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:data-flow".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
