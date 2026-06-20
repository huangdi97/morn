//! registry — Stores agent capability registrations and lookup indexes.

mod capability;
mod version;
mod manager;

pub use capability::Capability;
pub use version::{compare_versions, AgentTemplate};
pub use manager::Registry;
