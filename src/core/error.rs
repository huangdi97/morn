//! error — Shared error type for core and integration-facing code.

use std::error::Error;
use std::fmt;

pub type MornResult<T> = Result<T, MornError>;

#[derive(Debug, Clone, PartialEq)]
pub enum MornError {
    Config(String),
    Storage(String),
    Network(String),
    Budget(String),
    Security(String),
    Validation(String),
    NotFound(String),
    Conflict(String),
    Serialization(String),
    Io(String),
    Internal(String),
}

impl MornError {
    pub fn internal(message: impl Into<String>) -> Self {
        MornError::Internal(message.into())
    }
}

impl fmt::Display for MornError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, message) = match self {
            MornError::Config(message) => ("config", message),
            MornError::Storage(message) => ("storage", message),
            MornError::Network(message) => ("network", message),
            MornError::Budget(message) => ("budget", message),
            MornError::Security(message) => ("security", message),
            MornError::Validation(message) => ("validation", message),
            MornError::NotFound(message) => ("not found", message),
            MornError::Conflict(message) => ("conflict", message),
            MornError::Serialization(message) => ("serialization", message),
            MornError::Io(message) => ("io", message),
            MornError::Internal(message) => ("internal", message),
        };
        write!(f, "{} error: {}", kind, message)
    }
}

impl Error for MornError {}

impl From<String> for MornError {
    fn from(value: String) -> Self {
        MornError::Internal(value)
    }
}

impl From<&str> for MornError {
    fn from(value: &str) -> Self {
        MornError::Internal(value.to_string())
    }
}

impl From<std::io::Error> for MornError {
    fn from(value: std::io::Error) -> Self {
        MornError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for MornError {
    fn from(value: serde_json::Error) -> Self {
        MornError::Serialization(value.to_string())
    }
}

impl From<rusqlite::Error> for MornError {
    fn from(value: rusqlite::Error) -> Self {
        MornError::Storage(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn debug_format_matches_expected() {
        let err = MornError::Config("missing key".into());
        assert!(format!("{:?}", err).contains("Config"));
    }

    #[test]
    fn display_config_error() {
        let err = MornError::Config("missing key".into());
        assert_eq!(err.to_string(), "config error: missing key");
    }

    #[test]
    fn display_storage_error() {
        let err = MornError::Storage("disk full".into());
        assert_eq!(err.to_string(), "storage error: disk full");
    }

    #[test]
    fn display_network_error() {
        let err = MornError::Network("timeout".into());
        assert_eq!(err.to_string(), "network error: timeout");
    }

    #[test]
    fn display_budget_error() {
        let err = MornError::Budget("exceeded".into());
        assert_eq!(err.to_string(), "budget error: exceeded");
    }

    #[test]
    fn display_security_error() {
        let err = MornError::Security("unauthorized".into());
        assert_eq!(err.to_string(), "security error: unauthorized");
    }

    #[test]
    fn display_validation_error() {
        let err = MornError::Validation("invalid input".into());
        assert_eq!(err.to_string(), "validation error: invalid input");
    }

    #[test]
    fn display_not_found_error() {
        let err = MornError::NotFound("resource".into());
        assert_eq!(err.to_string(), "not found error: resource");
    }

    #[test]
    fn display_conflict_error() {
        let err = MornError::Conflict("duplicate".into());
        assert_eq!(err.to_string(), "conflict error: duplicate");
    }

    #[test]
    fn display_serialization_error() {
        let err = MornError::Serialization("parse failed".into());
        assert_eq!(err.to_string(), "serialization error: parse failed");
    }

    #[test]
    fn display_io_error() {
        let err = MornError::Io("permission denied".into());
        assert_eq!(err.to_string(), "io error: permission denied");
    }

    #[test]
    fn display_internal_error() {
        let err = MornError::Internal("bug".into());
        assert_eq!(err.to_string(), "internal error: bug");
    }

    #[test]
    fn error_trait_is_implemented() {
        let err = MornError::Internal("test".into());
        let trait_obj: &dyn Error = &err;
        assert!(trait_obj.source().is_none());
    }

    #[test]
    fn from_string_creates_internal() {
        let err: MornError = "oops".to_string().into();
        assert_eq!(err, MornError::Internal("oops".into()));
    }

    #[test]
    fn from_str_creates_internal() {
        let err: MornError = "oops".into();
        assert_eq!(err, MornError::Internal("oops".into()));
    }

    #[test]
    fn from_io_error_creates_io_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: MornError = io_err.into();
        assert!(matches!(err, MornError::Io(_)));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn from_serde_error_creates_serialization_variant() {
        let serde_err = serde_json::from_str::<serde_json::Value>("not valid json").unwrap_err();
        let err: MornError = serde_err.into();
        assert!(matches!(err, MornError::Serialization(_)));
    }

    #[test]
    fn internal_constructor_creates_internal() {
        let err = MornError::internal("something went wrong");
        assert_eq!(err, MornError::Internal("something went wrong".into()));
    }

    #[test]
    fn morn_result_works_with_ok() {
        let result: MornResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn morn_result_works_with_err() {
        let result: MornResult<i32> = Err(MornError::Internal("fail".into()));
        assert!(result.is_err());
    }
}
