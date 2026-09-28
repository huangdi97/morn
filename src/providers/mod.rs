//! providers — Morn Execution Plane (Phase 4).
//!
//! The Execution Plane is a heterogeneous executor set, not one runtime:
//! HarnessProviders (DSH / Pi / fake), a deterministic SolverProvider, a
//! HumanProvider, and Connector/ActionProviders (fake MES/CMMS in sandbox
//! mode only).
//!
//! BOUNDARY: business layers depend only on the Morn contracts in this module
//! (`HarnessProvider`, `SolverProvider`, `HumanProvider`,
//! `ConnectorProvider`, `CredentialProvider`). Harness session events are
//! Harness evidence — they never become Work truth. Agents never call real
//! production-write paths directly; side effects go Work → proposal →
//! Gateway/PEP → AuthorityDecision → Binding → Provider → Receipt.

pub mod connector;
pub mod connector_persistent;
pub mod credential;
pub mod harness;
pub mod harness_dsh;
pub mod harness_fake;
pub mod harness_pi;
pub mod human;
pub mod solver;

pub use connector::{ConnectorProvider, FakeCmmsConnector, FakeMesConnector};
pub use connector_persistent::PersistentFakeCmmsConnector;
pub use credential::{CredentialProvider, EnvCredentialProvider};
pub use harness::{
    HarnessCapabilities, HarnessEvent, HarnessEventKind, HarnessHealth, HarnessInput,
    HarnessProvider, HarnessSession, HarnessUsage,
};
pub use harness_dsh::DshHarnessProvider;
pub use harness_fake::FakeHarnessProvider;
pub use harness_pi::PiHarnessProvider;
pub use human::{HumanAssignment, HumanDecision, HumanProvider, HumanTaskKind};
pub use solver::{Assignment as SolverAssignment, SolverCandidate, SolverOutput, SolverProvider};
