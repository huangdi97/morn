//! sandbox — Sandboxed execution environments for untrusted agent code.
pub mod wasm;

#[cfg(feature = "vnc-sandbox")]
pub mod vnc;
