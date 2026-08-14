//! Version type (semver-like) used across Morn records.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A semantic version `major.minor.patch`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn v1() -> Self {
        Self::new(1, 0, 0)
    }

    pub fn next_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    pub fn next_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn next_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::validation(format!("invalid version: {s}")));
        }
        let parse = |i: usize| {
            parts[i]
                .parse::<u32>()
                .map_err(|_| Error::validation(format!("invalid version: {s}")))
        };
        Ok(Version::new(parse(0)?, parse(1)?, parse(2)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_roundtrip() {
        let v: Version = "1.3.0".parse().unwrap();
        assert_eq!(v, Version::new(1, 3, 0));
        assert_eq!(v.to_string(), "1.3.0");
        assert_eq!(v.next_patch(), Version::new(1, 3, 1));
        assert_eq!(v.next_minor(), Version::new(1, 4, 0));
    }

    #[test]
    fn version_rejects_bad_input() {
        assert!("1.3".parse::<Version>().is_err());
        assert!("x.y.z".parse::<Version>().is_err());
    }
}
