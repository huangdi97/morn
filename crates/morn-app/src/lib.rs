//! Morn application services + HTTP API shared by all four product surfaces.

pub mod api;
pub mod app;

pub use app::{AppInner, AppState};
pub use api::{router, AppError};
