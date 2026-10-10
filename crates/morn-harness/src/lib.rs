//! Harness fabric: HarnessSpec/Binding, CapabilityScope, ExecutionEvent/Receipt,
//! providers (MornNative + DeepSeekHarness + Pi boundaries) and contract suite.

pub mod binding;
pub mod context;
pub mod contract;
pub mod dsh_sdk;
pub mod event;
pub mod intelligence;
pub mod neutrality;
pub mod pi;
pub mod pi_rpc;
pub mod provider;
pub mod receipt;
pub mod scope;
pub mod smoke;
pub mod spec;
mod subprocess_env;
mod subprocess_wire;

pub use binding::{HarnessBinding, RuntimeBinding};
pub use context::RuntimeContext;
pub use contract::run_provider_contract;
pub use dsh_sdk::{
    assistant_text_from_session_event, DshNotification, DshSdkConfig, DshSdkRunResult,
    DshSdkStdioClient, DSH_METHOD_INITIALIZE, DSH_METHOD_SESSION_PROMPT, DSH_METHOD_SHUTDOWN,
};
pub use event::{ExecutionEvent, ExecutionEventKind};
pub use intelligence::{
    run_intelligence_conformance, IntelligenceProvider, IntelligenceRequest, IntelligenceResult,
    RuleIntelligence, SolverIntelligence,
};
pub use neutrality::{run_harness_neutrality, HarnessNeutralityReport};
pub use pi::{PiHarnessProvider, PiMode};
pub use pi_rpc::{
    PiPromptRun, PiRpcClient, PiRpcConfig, PiRpcEvent, PiRpcResponse, PI_COMMAND_ABORT,
    PI_COMMAND_GET_STATE, PI_COMMAND_PROMPT, PI_EVENT_AGENT_SETTLED,
};
pub use provider::{
    DeepSeekHarnessProvider, HarnessProvider, HarnessProviderFeatures, HarnessRuntimeHealth,
    HarnessRuntimeHealthState, HarnessSession, MornNativeHarness, ProviderHandle,
    HARNESS_RUNTIME_HEALTH_LEASE_MS,
};
pub use receipt::{
    ExecutionReceipt, ExecutorOutcomeDisposition, ExecutorOutcomeReconciliation,
    ExecutorOutcomeReconciliationAuthorization, ExecutorOutcomeReconciliationId,
};
pub use scope::{CapabilityScope, ScopeKind};
pub use smoke::{run_harness_smoke, HarnessSmokeReport};
pub use spec::{HarnessSpec, HarnessVersion};
