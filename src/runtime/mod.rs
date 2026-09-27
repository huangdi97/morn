//! runtime — Morn Composition Runtime adapters.
//!
//! Per the v11.5 master (§8.2): implementation coupling to composition
//! runtimes is confined to this module tree. Domain, spec, and control-plane
//! code must not import composition runtime types directly.

pub mod cordis;
