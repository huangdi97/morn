//! slots — Morn service slots as Cordis Service contracts.
//!
//! Each slot is one `Service` implementation with a stable contract name
//! (`morn.<slot>`); providers publish into the slot via the host, and
//! consumers resolve the slot by contract instead of importing a concrete
//! provider. Slot payloads are composition metadata — they are NOT business
//! truth: a provider visible in a slot is not "capability qualified" and not
//! "site admitted" (master §7.2).

use cordis::Service;
use serde::{Deserialize, Serialize};

/// One provider registered in a slot (composition metadata).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub provider_id: String,
    pub version: String,
    /// e.g. harness / solver / connector / environment
    pub kind: String,
}

impl ProviderDescriptor {
    pub fn new(provider_id: &str, version: &str, kind: &str) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            version: version.to_string(),
            kind: kind.to_string(),
        }
    }
}

/// Harness provider metadata (runtime composition view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessProviderInfo {
    pub descriptor: ProviderDescriptor,
    /// Harness contracts the provider can run, e.g. ["dsh", "pi"]
    pub harness_kinds: Vec<String>,
}

/// Capability provider metadata (composition view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityProviderInfo {
    pub descriptor: ProviderDescriptor,
    /// Capability ids the provider can execute
    pub capabilities: Vec<String>,
}

macro_rules! morn_slot {
    ($slot:ident, $name:literal, $payload:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Default)]
        pub struct $slot {
            pub providers: Vec<$payload>,
        }

        impl $slot {
            /// Inherent contract-name constant (usable without importing the
            /// upstream Service trait).
            pub const CONTRACT: &'static str = $name;

            pub fn with_provider(mut self, provider: $payload) -> Self {
                self.providers.push(provider);
                self
            }

            pub fn contract_name(&self) -> &'static str {
                Self::CONTRACT
            }
        }

        impl Service for $slot {
            const NAME: &'static str = $name;
        }
    };
}

morn_slot!(
    CapabilitiesSlot,
    "morn.capabilities",
    CapabilityProviderInfo,
    "Capability providers (composition view; not qualification)."
);
morn_slot!(
    HarnessesSlot,
    "morn.harnesses",
    HarnessProviderInfo,
    "Harness providers: DSH / Pi / future providers (composition view)."
);
morn_slot!(
    SolversSlot,
    "morn.solvers",
    ProviderDescriptor,
    "Solver providers: deterministic optimization (composition view)."
);
morn_slot!(
    ExecutionEnvironmentsSlot,
    "morn.executionEnvironments",
    ProviderDescriptor,
    "Execution environment providers: local-process / container / future (composition view)."
);
morn_slot!(
    ConnectorsSlot,
    "morn.connectors",
    ProviderDescriptor,
    "Connector / action providers: ERP / MES / CMMS / fake (composition view)."
);
morn_slot!(
    PolicySlot,
    "morn.policy",
    ProviderDescriptor,
    "Policy decision providers: OPA / Cedar / reference (composition view)."
);
morn_slot!(
    IdentitySlot,
    "morn.identity",
    ProviderDescriptor,
    "Identity providers: workload identity / IAM (composition view)."
);
morn_slot!(
    TelemetrySlot,
    "morn.telemetry",
    ProviderDescriptor,
    "Telemetry providers: OTel exporters (composition view)."
);

/// The full set of Morn service slot contract names (conformance surface).
pub const SLOT_NAMES: [&str; 8] = [
    CapabilitiesSlot::CONTRACT,
    HarnessesSlot::CONTRACT,
    SolversSlot::CONTRACT,
    ExecutionEnvironmentsSlot::CONTRACT,
    ConnectorsSlot::CONTRACT,
    PolicySlot::CONTRACT,
    IdentitySlot::CONTRACT,
    TelemetrySlot::CONTRACT,
];

/// Slot errors surfaced through the Morn contract (not raw Cordis errors).
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceSlotError {
    /// The slot has no visible provider right now.
    Unavailable { slot: String },
}

impl std::fmt::Display for ServiceSlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceSlotError::Unavailable { slot } => {
                write!(f, "service slot '{slot}' unavailable")
            }
        }
    }
}

impl std::error::Error for ServiceSlotError {}
