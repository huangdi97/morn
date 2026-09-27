//! cordis — Morn's reference composition runtime adapter.
//!
//! This is the ONLY module tree directly coupled to the Cordis API (v11.5
//! master §8.2). Upstream: `cordis-rs` =0.8.39 (facade over cordis-core
//! 0.3.24), github.com/dshbox/cordis-rs, MIT, a Rust port of DeepSeek Harness
//! Cordis 4.x. Pre-1.0 upstream: breaking changes land as minor bumps, so
//! version upgrades are explicit code changes, never silent.
//!
//! Mapping (master §8):
//! ```text
//! Morn service slot        <-> Cordis Service (const NAME)
//! Morn runtime scope       <-> Cordis Context / isolate axis
//! provider lifecycle       <-> Fiber / effect lifecycle
//! ```
//!
//! FORBIDDEN identifications (master §3.1 / §7.2 — enforced by tests):
//! ```text
//! Fiber == Work                           NO
//! Effect disposer == business rollback    NO
//! service active == capability qualified  NO
//! provider ready == site admitted         NO
//! ```
//!
//! Work / Receipt / Outcome durable business truth stays in the Control Plane
//! (`crate::control`); Cordis service state is runtime composition truth only.

pub mod host;
pub mod slots;

pub use host::CompositionHost;
pub use slots::{
    CapabilitiesSlot, CapabilityProviderInfo, ConnectorsSlot, ExecutionEnvironmentsSlot,
    HarnessProviderInfo, HarnessesSlot, IdentitySlot, PolicySlot, ProviderDescriptor,
    ServiceSlotError, SolversSlot, TelemetrySlot, SLOT_NAMES,
};

/// Exact pinned upstream versions, recorded per Goal §7.1 (Spike evidence).
pub const UPSTREAM_FACADE_VERSION: &str = "0.8.39";
pub const UPSTREAM_CORE_VERSION: &str = "0.3.24";
pub const UPSTREAM_LICENSE: &str = "MIT";
pub const UPSTREAM_SOURCE: &str = "https://github.com/dshbox/cordis-rs";
