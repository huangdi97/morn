//! registry — Stores agent capability registrations and lookup indexes.

mod capability;
mod manager;
mod version;

pub use capability::Capability;
pub use manager::Registry;
pub use version::{compare_versions, AgentTemplate};
