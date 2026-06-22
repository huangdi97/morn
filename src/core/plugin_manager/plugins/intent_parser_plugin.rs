use std::sync::Arc;
use crate::core::intent_parser::IntentParser;
use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

pub struct IntentParserPlugin(pub Option<Arc<IntentParser>>);

impl Default for IntentParserPlugin {
    fn default() -> Self { Self::new() }
}

impl IntentParserPlugin {
    pub fn new() -> Self { Self(None) }
}

impl MornPlugin for IntentParserPlugin {
    fn id(&self) -> &str { "morn:intent-parser" }
    fn deps(&self) -> Vec<&str> { vec!["morn:model-router"] }
    fn priority(&self) -> i32 { 150 }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        let parser = Arc::new(IntentParser);
        ctx.register("morn:intent-parser", parser.clone());
        self.0 = Some(parser);
        Ok(())
    }

    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        ctx.get::<Arc<IntentParser>>("morn:intent-parser").ok_or_else(|| {
            PluginError::ActivateFailed("morn:intent-parser".into(), "not registered".into())
        })?;
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.0 = None;
        Ok(())
    }
}