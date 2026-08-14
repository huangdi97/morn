//! SQLite persistence adapter for Morn records.
//!
//! The kernel/domain interfaces are database-agnostic; this crate is a local
//! SQLite adapter (future PostgreSQL can replace it).

pub mod store;

pub use store::MornStore;
pub use morn_kernel::error::Error as StoreError;

