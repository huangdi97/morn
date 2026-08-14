//! Morn Kernel: the stable semantic kernel. Small, low-coupling, domain-agnostic.
//! Owns Identity / Workspace / Policy / Approval / Ledger / Lifecycle semantics.

pub mod approval;
pub mod error;
pub mod identity;
pub mod ids;
pub mod ledger;
pub mod lifecycle;
pub mod policy;
pub mod status;
pub mod time;
pub mod version;
pub mod workspace;

pub use error::{Error, Result};
