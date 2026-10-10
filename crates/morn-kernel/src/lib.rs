//! Morn Kernel: compact domain-neutral primitives shared by the reference
//! implementation. Business semantics evolve through versioned protocol
//! contracts; providers must not silently redefine an active contract.

pub mod approval;
pub mod contracts;
pub mod error;
pub mod event_envelope;
pub mod event_semantics;
pub mod execution_guarantee;
pub mod history;
pub mod identity;
pub mod ids;
pub mod ledger;
pub mod lifecycle;
pub mod policy;
pub mod protocol;
pub mod status;
pub mod telemetry;
pub mod time;
pub mod version;
pub mod workspace;

pub use contracts::{
    Compatibility, ContractCompatibility, ContractSnapshot, DeprecationPolicy, SEMANTIC_CONTRACT_V1,
};
pub use error::{Error, Result};
pub use event_envelope::EventEnvelope;
pub use event_semantics::{EventSemanticClass, EventSemanticDescriptor};
pub use execution_guarantee::{ExecutionClass, ExecutionGuarantee};
pub use history::{HistoricalFact, HistoricalFactId, HistoricalFactLog, HistoricalFactStatus};
pub use protocol::{
    HistoryMutation, HistoryMutationKind, ProtocolSnapshot, PublishedContractRef,
    MORN_PROTOCOL_V11_5,
};
pub use telemetry::{
    MornTelemetryOperation, TelemetryContext, TelemetryRecord, ATTR_ACCEPTANCE_ID, ATTR_ATTEMPT_ID,
    ATTR_AUTHORITY_DECISION_ID, ATTR_BINDING_ID, ATTR_CAPABILITY_DIGEST, ATTR_CAPABILITY_ID,
    ATTR_EVIDENCE_CLASS, ATTR_EXECUTION_ENVIRONMENT_ID, ATTR_OUTCOME_ID, ATTR_PROFILE_ID,
    ATTR_PROVIDER_ID, ATTR_RECONCILIATION_ID, ATTR_RUNTIME_ID, ATTR_SITE_ID, ATTR_WORK_GENERATION,
    ATTR_WORK_ID, ATTR_WORK_TYPE,
};
