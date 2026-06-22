use crate::core::code_tool::CodeToolExecutor;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};
use std::sync::Arc;

pub struct CodeToolPlugin(pub Option<Arc<CodeToolExecutor>>);

impl Default for CodeToolPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeToolPlugin {
    pub fn new() -> Self {
        Self(None)
    }
}

impl MornPlugin for CodeToolPlugin {
    fn id(&self) -> &str {
        "morn:code-tool"
    }
    fn deps(&self) -> Vec<&str> {
        vec!["morn:sandbox"]
    }
    fn priority(&self) -> i32 {
        90
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let executor = Arc::new(CodeToolExecutor::new());
        ctx.register("morn:code-tool", executor.clone());
        self.0 = Some(executor);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<CodeToolExecutor>>("morn:code-tool")
            .ok_or_else(|| {
                PluginError::ActivateFailed("morn:code-tool".into(), "not registered".into())
            })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}
