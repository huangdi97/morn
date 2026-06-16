//! 检查点枚举与结果类型 — Checkpoint, CheckResult, InjectionRisk
//! 定义双 LLM 安全检查点的阶段、结果和风险等级。

use crate::core::error::MornError;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Checkpoint {
    Auth,
    ParamValidate,
    ContentSanitize,
    Permission,
    Audit,
    Route,
}

impl Checkpoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            Checkpoint::Auth => "authentication",
            Checkpoint::ParamValidate => "parameter_validation",
            Checkpoint::ContentSanitize => "content_sanitization",
            Checkpoint::Permission => "permission_check",
            Checkpoint::Audit => "audit_log",
            Checkpoint::Route => "route_release",
        }
    }

    pub fn order() -> Vec<Checkpoint> {
        vec![
            Checkpoint::Auth,
            Checkpoint::ParamValidate,
            Checkpoint::ContentSanitize,
            Checkpoint::Permission,
            Checkpoint::Audit,
            Checkpoint::Route,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CheckResult {
    Pass,
    Flag(String),
    Block(String),
}

impl CheckResult {
    pub fn is_blocked(&self) -> bool {
        matches!(self, CheckResult::Block(_))
    }

    pub fn is_flagged(&self) -> bool {
        matches!(self, CheckResult::Flag(_))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InjectionRisk {
    None,
    Low(String),
    Medium(String),
    High(String),
}
