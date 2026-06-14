use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum CommandError {
    NotFound(String),
    Unauthorized(String),
    InvalidInput(String),
    Internal(String),
    Network(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NotFound(msg) => write!(f, "Not found: {}", msg),
            CommandError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            CommandError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CommandError::Internal(msg) => write!(f, "Internal error: {}", msg),
            CommandError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}
