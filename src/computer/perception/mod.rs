//! perception — Provides screen and environment perception for computer operations.
use crate::core::error::MornError;
pub mod accessibility;
pub mod ocr;
pub mod screenshot;

pub use accessibility::accessibility_tree;
pub use ocr::ocr;
pub use screenshot::{analyze_screen, pixel_screenshot};
