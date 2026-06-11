//! 双 LLM 安全校验模块 — 通过主/副 LLM 判断和检查点序列进行安全检查。
//! 子模块: `checkpoints` (枚举与结果类型), `engine` (DualLlmGuard 实现)。

mod checkpoints;
mod engine;

pub use checkpoints::{CheckResult, Checkpoint, InjectionRisk};
pub use engine::{
    DualLlmExecutorDecision, DualLlmGuard, DualLlmGuardDecision, DualLlmJudgeDecision, DualLlmLog,
    LlmJudgeFn,
};
