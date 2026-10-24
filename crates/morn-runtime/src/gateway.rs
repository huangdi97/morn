//! Compatibility re-export for the governed World action boundary.
//!
//! The implementation lives in `morn-world::gateway` so the raw canonical
//! World commit primitive can remain crate-private and cannot be called by
//! arbitrary runtime/provider code.

pub use morn_world::gateway::*;
