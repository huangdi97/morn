//! Harness fabric: HarnessSpec/Binding, CapabilityScope, ExecutionEvent/Receipt,
//! providers (MornNative + DeepSeekHarness boundary) and contract suite.

pub mod binding;
pub mod context;
pub mod contract;
pub mod event;
pub mod provider;
pub mod receipt;
pub mod scope;
pub mod spec;

pub use binding::{HarnessBinding, RuntimeBinding};
pub use context::RuntimeContext;
pub use contract::run_provider_contract;
pub use event::{ExecutionEvent, ExecutionEventKind};
pub use provider::{DeepSeekHarnessProvider, HarnessProvider, HarnessSession, MornNativeHarness, ProviderHandle};
pub use receipt::ExecutionReceipt;
pub use scope::{CapabilityScope, ScopeKind};
pub use spec::{HarnessSpec, HarnessVersion};
