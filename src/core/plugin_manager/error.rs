use crate::core::error::MornError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Invalid manifest in {0}: {1}")]
    InvalidManifest(String, String),
    #[error("Scaffold error: {0}")]
    Scaffold(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}
