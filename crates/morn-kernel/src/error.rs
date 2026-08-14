//! Structured errors for the Morn kernel.

use thiserror::Error;

/// Unified structured error type for Morn domain operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("not authorized: {0}")]
    NotAuthorized(String),
    #[error("invalid state transition: {0}")]
    InvalidState(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("external provider error: {0}")]
    External(String),
}

impl Error {
    pub fn not_found<T: std::fmt::Display>(what: T) -> Self {
        Error::NotFound(what.to_string())
    }

    pub fn validation<T: std::fmt::Display>(msg: T) -> Self {
        Error::Validation(msg.to_string())
    }

    pub fn forbidden<T: std::fmt::Display>(msg: T) -> Self {
        Error::Forbidden(msg.to_string())
    }

    pub fn invalid_state<T: std::fmt::Display>(msg: T) -> Self {
        Error::InvalidState(msg.to_string())
    }

    pub fn not_authorized<T: std::fmt::Display>(msg: T) -> Self {
        Error::NotAuthorized(msg.to_string())
    }

    pub fn conflict<T: std::fmt::Display>(msg: T) -> Self {
        Error::Conflict(msg.to_string())
    }

    pub fn internal<T: std::fmt::Display>(msg: T) -> Self {
        Error::Internal(msg.to_string())
    }

    pub fn external<T: std::fmt::Display>(msg: T) -> Self {
        Error::External(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
