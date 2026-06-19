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
    #[error("Plugin order error: {0}")]
    OrderError(String),
    #[error("Failed to load plugin '{0}': {1}")]
    LoadFailed(String, String),
    #[error("Failed to activate plugin '{0}': {1}")]
    ActivateFailed(String, String),
    #[error("Plugin type mismatch: expected {0}, got {1}")]
    PluginTypeMismatch(String, String),
    #[error("{0}")]
    Other(String),
}

/// Errors related to plugin dependency ordering.
#[derive(Debug, Error)]
pub enum PluginOrderError {
    #[error("Cycle detected among plugins: {0:?}")]
    CycleDetected(Vec<String>),
    #[error("Dependency '{1}' required by plugin '{0}' was not found")]
    MissingDependency(String, String),
}
