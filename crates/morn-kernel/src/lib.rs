//! Morn Kernel: compact domain-neutral primitives shared by the reference
//! implementation. Business semantics evolve through versioned protocol
//! contracts; providers must not silently redefine an active contract.

pub mod approval;
pub mod contracts;
pub mod error;
pub mod event_envelope;
pub mod history;
pub mod identity;
pub mod ids;
pub mod ledger;
pub mod lifecycle;
pub mod policy;
pub mod protocol;
pub mod status;
pub mod time;
pub mod telemetry;
pub mod version;
pub mod workspace;

pub use contracts::{
    Compatibility, ContractCompatibility, ContractSnapshot, DeprecationPolicy, SEMANTIC_CONTRACT_V1,
};
pub use error::{Error, Result};
pub use event_envelope::EventEnvelope;
pub use history::{HistoricalFact, HistoricalFactId, HistoricalFactLog, HistoricalFactStatus};
pub use telemetry::TelemetryContext;
pub use protocol::{
    HistoryMutation, HistoryMutationKind, ProtocolSnapshot, PublishedContractRef,
    MORN_PROTOCOL_V11_5,
};
