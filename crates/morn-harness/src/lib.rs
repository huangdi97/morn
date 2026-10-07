//! Harness fabric: HarnessSpec/Binding, CapabilityScope, ExecutionEvent/Receipt,
//! providers (MornNative + DeepSeekHarness + Pi boundaries) and contract suite.

pub mod binding;
pub mod context;
pub mod contract;
pub mod event;
pub mod intelligence;
pub mod neutrality;
pub mod pi;
pub mod provider;
pub mod receipt;
pub mod scope;
pub mod smoke;
pub mod spec;

pub use binding::{HarnessBinding, RuntimeBinding};
pub use context::RuntimeContext;
pub use contract::run_provider_contract;
pub use event::{ExecutionEvent, ExecutionEventKind};
pub use intelligence::{
    run_intelligence_conformance, IntelligenceProvider, IntelligenceRequest, IntelligenceResult,
    RuleIntelligence, SolverIntelligence,
};
pub use neutrality::{run_harness_neutrality, HarnessNeutralityReport};
pub use pi::{PiHarnessProvider, PiMode};
pub use provider::{
    DeepSeekHarnessProvider, HarnessProvider, HarnessSession, MornNativeHarness, ProviderHandle,
};
pub use receipt::ExecutionReceipt;
pub use scope::{CapabilityScope, ScopeKind};
pub use smoke::{run_harness_smoke, HarnessSmokeReport};
pub use spec::{HarnessSpec, HarnessVersion};
