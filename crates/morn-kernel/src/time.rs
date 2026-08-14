//! Timestamps as UTC instants with millisecond precision.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A UTC timestamp with millisecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_millis(ms: i64) -> Self {
        Self(DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now))
    }

    pub fn millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
