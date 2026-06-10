//! 双 LLM 安全引擎 — DualLlmGuard 结构与安全检查逻辑
//! 通过主/副 LLM 判断和检查点序列对输入进行安全检查。

use crate::bridge::chat_agent::ChatAgent;
use crate::core::security::{AuditLog, SecurityGuard, SecurityProfile};

use super::checkpoints::{CheckResult, Checkpoint, InjectionRisk};

pub type LlmJudgeFn = Box<dyn Fn() -> Result<String, String> + Send + Sync>;

#[allow(dead_code)] /* 预留：部分字段为架构保留 */
pub struct DualLlmGuard {
    primary: Option<ChatAgent>,
    secondary: Option<ChatAgent>,
    primary_llm: Option<LlmJudgeFn>,
    secondary_llm: Option<LlmJudgeFn>,
    checkpoints: Vec<Checkpoint>,
    enabled: bool,
    pub(crate) log: Vec<DualLlmLog>,
    pub(crate) security_guard: Option<SecurityGuard>,
    pub(crate) audit_log: Option<AuditLog>,
    pub(crate) security_profile: Option<SecurityProfile>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DualLlmLog {
    pub timestamp: String,
    pub input_preview: String,
    pub risk: String,
    pub check_results: Vec<CheckResult>,
    pub allowed: bool,
}

impl DualLlmGuard {
    pub fn new(primary: Option<ChatAgent>, secondary: Option<ChatAgent>) -> Self {
        DualLlmGuard {
            primary,
            secondary,
            primary_llm: None,
            secondary_llm: None,
            checkpoints: Checkpoint::order(),
            enabled: true,
            log: Vec::new(),
            security_guard: Some(SecurityGuard::new()),
            audit_log: Some(AuditLog::new()),
            security_profile: Some(SecurityProfile::default()),
        }
    }

    pub fn with_llm_checks(primary_llm: LlmJudgeFn, secondary_llm: LlmJudgeFn) -> Self {
        DualLlmGuard {
            primary: None,
            secondary: None,
            primary_llm: Some(primary_llm),
            secondary_llm: Some(secondary_llm),
            checkpoints: Checkpoint::order(),
            enabled: true,
            log: Vec::new(),
            security_guard: Some(SecurityGuard::new()),
            audit_log: Some(AuditLog::new()),
            security_profile: Some(SecurityProfile::default()),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    pub fn inspect(&mut self, input: &str, params: &serde_json::Value) -> CheckResult {
        self.check(input, params)
    }

    pub fn check(&mut self, input: &str, params: &serde_json::Value) -> CheckResult {
        if !self.enabled {
            return CheckResult::Pass;
        }

        let mut all_results = Vec::new();
        if let Some(primary_result) = self.run_primary_llm_check() {
            let suspicious = primary_result.is_blocked() || primary_result.is_flagged();
            all_results.push(primary_result);
            if suspicious {
                let secondary_result = self.run_secondary_check(input);
                let secondary_suspicious =
                    secondary_result.is_blocked() || secondary_result.is_flagged();
                all_results.push(secondary_result);
                if secondary_suspicious {
                    return self.finish_check(
                        input,
                        all_results,
                        InjectionRisk::High("primary and secondary LLM judgments matched".into()),
                    );
                }
            }
        }

        let checkpoints = self.checkpoints.clone();
        for checkpoint in &checkpoints {
            let result = self.run_checkpoint_mut(checkpoint, input, params);
            let blocked = result.is_blocked();
            all_results.push(result);
            if blocked {
                break;
            }
        }

        let blocked = all_results.iter().any(|r| r.is_blocked());
        let flagged = all_results.iter().any(|r| r.is_flagged());

        let risk_level = if blocked {
            InjectionRisk::High(input.to_string())
        } else if flagged {
            let secondary_check = self.run_secondary_check(input);
            match secondary_check {
                CheckResult::Block(msg) => InjectionRisk::High(msg),
                CheckResult::Flag(msg) => InjectionRisk::Medium(msg),
                CheckResult::Pass => InjectionRisk::Low("flagged by primary check".into()),
            }
        } else {
            InjectionRisk::None
        };

        self.finish_check(input, all_results, risk_level)
    }

    fn finish_check(
        &mut self,
        input: &str,
        all_results: Vec<CheckResult>,
        risk_level: InjectionRisk,
    ) -> CheckResult {
        let allowed = !matches!(risk_level, InjectionRisk::High(_));
        let risk = match risk_level {
            InjectionRisk::High(_) => "high",
            InjectionRisk::Medium(_) => "medium",
            InjectionRisk::Low(_) => "low",
            InjectionRisk::None => "none",
        };
        self.log.push(DualLlmLog {
            timestamp: chrono::Utc::now().to_rfc3339(),
            input_preview: input.chars().take(100).collect(),
            risk: risk.to_string(),
            check_results: all_results,
            allowed,
        });

        match risk_level {
            InjectionRisk::High(msg) => CheckResult::Block(msg),
            InjectionRisk::Medium(msg) => CheckResult::Flag(msg),
            InjectionRisk::Low(msg) => CheckResult::Flag(msg),
            InjectionRisk::None => CheckResult::Pass,
        }
    }

    fn run_primary_llm_check(&self) -> Option<CheckResult> {
        self.primary_llm
            .as_ref()
            .map(|judge| Self::parse_llm_judgment(judge()))
    }

    fn run_checkpoint_mut(
        &mut self,
        checkpoint: &Checkpoint,
        input: &str,
        _params: &serde_json::Value,
    ) -> CheckResult {
        if matches!(checkpoint, Checkpoint::Permission) {
            let blocked = self.security_guard.as_ref()
                .map(|g| g.is_allowed(input, _params).is_err())
                .unwrap_or(false);
            return if blocked { CheckResult::Block("Permission denied by security policy".into()) } else { CheckResult::Pass };
        }

        if matches!(checkpoint, Checkpoint::Audit) {
            if let Some(ref mut alog) = self.audit_log {
                let agent = self.security_profile.as_ref()
                    .map(|p| p.agent_id.as_str()).unwrap_or("unknown");
                alog.append(agent, "dual_llm_check", input);
            }
            return CheckResult::Pass;
        }

        if matches!(checkpoint, Checkpoint::Route) {
            let info = self.security_profile.as_ref()
                .map(|p| (p.agent_id.clone(), p.sandbox_level));
            if let Some((agent_id, level)) = info {
                if level >= 4 {
                    return CheckResult::Flag(format!("Agent '{}' at level {} needs elevated routing", agent_id, level));
                }
            }
            return CheckResult::Pass;
        }

        match checkpoint {
            Checkpoint::Auth => {
                if input.contains("api_key")
                    || input.contains("password")
                    || input.contains("secret")
                    || input.contains("token")
                {
                    return CheckResult::Flag("Input may contain sensitive credentials".into());
                }
                CheckResult::Pass
            }
            Checkpoint::ParamValidate => {
                if input.len() > 10000 {
                    return CheckResult::Flag("Input exceeds 10000 character limit".into());
                }
                CheckResult::Pass
            }
            Checkpoint::ContentSanitize => {
                let dangerous = [
                    "DROP TABLE",
                    "DELETE FROM",
                    "rm -rf",
                    "format ",
                    "shutdown",
                    "sudo ",
                    "chmod 777",
                    "> /dev/",
                    "| sh",
                    "`command`",
                    "system(",
                    "exec(",
                    "eval(",
                    "os.system",
                    "ignore previous",
                    "ignore all",
                    "forget your",
                    "act as if",
                    "you are now",
                    "pretend to",
                    "from now on",
                    "override",
                    "disregard",
                    "you must ",
                    "you have to ",
                ];
                let upper = input.to_uppercase();
                for pattern in &dangerous {
                    if upper.contains(&pattern.to_uppercase()) {
                        return CheckResult::Block(format!(
                            "Content contains dangerous pattern: {}",
                            pattern
                        ));
                    }
                }
                CheckResult::Pass
            }
            Checkpoint::Permission | Checkpoint::Audit | Checkpoint::Route => {
                unreachable!("already handled by early return")
            }
        }
    }

    fn run_secondary_check(&self, input: &str) -> CheckResult {
        if let Some(judge) = self.secondary_llm.as_ref() {
            return Self::parse_llm_judgment(judge());
        }

        // fallback: detect dangerous patterns (injection, command, override)
        let dangerous = [
            "DROP TABLE",
            "DELETE FROM",
            "rm -rf",
            "sudo ",
            "chmod 777",
            "> /dev/",
            "| sh",
            "`command`",
            "system(",
            "exec(",
            "eval(",
            "os.system",
            "ignore previous",
            "ignore all",
            "forget your",
            "act as if",
            "you are now",
            "pretend to",
            "from now on",
            "override",
            "disregard",
            "you must ",
            "you have to ",
        ];
        let upper = input.to_uppercase();
        for pattern in &dangerous {
            if upper.contains(&pattern.to_uppercase()) {
                return CheckResult::Block(format!(
                    "Secondary check: dangerous pattern detected: {}",
                    pattern
                ));
            }
        }
        CheckResult::Pass
    }

    fn parse_llm_judgment(result: Result<String, String>) -> CheckResult {
        match result {
            Ok(text) => {
                let lower = text.trim().to_lowercase();
                if lower == "block"
                    || lower == "suspicious"
                    || lower.starts_with("block")
                    || lower.starts_with("suspicious")
                {
                    CheckResult::Block(text)
                } else if lower == "flag"
                    || lower == "review"
                    || lower.starts_with("flag")
                    || lower.starts_with("review")
                {
                    CheckResult::Flag(text)
                } else {
                    CheckResult::Pass
                }
            }
            Err(err) => CheckResult::Flag(format!("LLM judgment failed: {}", err)),
        }
    }

    pub fn get_log(&self) -> &[DualLlmLog] {
        &self.log
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;