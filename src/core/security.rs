//! 四层安全宪法——守卫智能体动作的安全分级机制。
//!
//! 安全级别（L1→L4 逐层放宽）：
//! - L1HardBlocked: 硬拦截，属于危险操作（如格式化磁盘、删除系统文件）
//! - L2NeedApproval: 需审批，高风险操作需用户确认（如执行 shell、写工作区外文件）
//! - L3NeedNotify: 需通知，操作执行时发送通知（如读工作区外文件、访问未注册域名）
//! - L4Free: 自由执行，低风险操作（如聊天、搜索、读工作区内文件）
//!
//! Dual-LLM 集成: 策略匹配由 SecurityGuard 的规则引擎完成，不依赖 LLM 判定。

use serde_json::Value;
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    L1HardBlocked,
    L2NeedApproval,
    L3NeedNotify,
    L4Free,
}

impl SecurityLevel {
    /// 将安全级别转为字符串标识。
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityLevel::L1HardBlocked => "L1HardBlocked",
            SecurityLevel::L2NeedApproval => "L2NeedApproval",
            SecurityLevel::L3NeedNotify => "L3NeedNotify",
            SecurityLevel::L4Free => "L4Free",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub level: SecurityLevel,
    pub pattern: String,
    pub description: String,
}

pub struct SecurityGuard {
    policies: Vec<SecurityPolicy>,
    pub block_enabled: bool,
    pub notify_enabled: bool,
}

impl SecurityGuard {
    /// 创建 SecurityGuard 实例，内置默认安全策略。
    pub fn new() -> Self {
        let policies = vec![
            SecurityPolicy {
                name: "format_disk".to_string(),
                level: SecurityLevel::L1HardBlocked,
                pattern: "format_disk".to_string(),
                description: "Format or wipe disk drives".to_string(),
            },
            SecurityPolicy {
                name: "delete_system_file".to_string(),
                level: SecurityLevel::L1HardBlocked,
                pattern: "delete_system_file".to_string(),
                description: "Delete critical system files".to_string(),
            },
            SecurityPolicy {
                name: "modify_system_registry".to_string(),
                level: SecurityLevel::L1HardBlocked,
                pattern: "modify_system_registry".to_string(),
                description: "Modify system registry entries".to_string(),
            },
            SecurityPolicy {
                name: "execute_shell".to_string(),
                level: SecurityLevel::L2NeedApproval,
                pattern: "execute_shell".to_string(),
                description: "Execute arbitrary shell commands".to_string(),
            },
            SecurityPolicy {
                name: "write_outside_workspace".to_string(),
                level: SecurityLevel::L2NeedApproval,
                pattern: "write_outside_workspace".to_string(),
                description: "Write files outside workspace directory".to_string(),
            },
            SecurityPolicy {
                name: "read_outside_workspace".to_string(),
                level: SecurityLevel::L3NeedNotify,
                pattern: "read_outside_workspace".to_string(),
                description: "Read files outside workspace directory".to_string(),
            },
            SecurityPolicy {
                name: "network_unregistered_domain".to_string(),
                level: SecurityLevel::L3NeedNotify,
                pattern: "network_unregistered_domain".to_string(),
                description: "Access unregistered network domains".to_string(),
            },
            SecurityPolicy {
                name: "chat".to_string(),
                level: SecurityLevel::L4Free,
                pattern: "chat".to_string(),
                description: "Chat with user".to_string(),
            },
            SecurityPolicy {
                name: "search".to_string(),
                level: SecurityLevel::L4Free,
                pattern: "search".to_string(),
                description: "Search for information".to_string(),
            },
            SecurityPolicy {
                name: "read_workspace_file".to_string(),
                level: SecurityLevel::L4Free,
                pattern: "read_workspace_file".to_string(),
                description: "Read files within workspace".to_string(),
            },
            SecurityPolicy {
                name: "call_registered_api".to_string(),
                level: SecurityLevel::L4Free,
                pattern: "call_registered_api".to_string(),
                description: "Call registered API endpoints".to_string(),
            },
        ];
        SecurityGuard {
            policies,
            block_enabled: true,
            notify_enabled: true,
        }
    }

    /// 检查动作的安全级别。
    pub fn check(&self, action: &str, _params: &Value) -> SecurityLevel {
        for policy in &self.policies {
            if action.contains(&policy.pattern) || policy.pattern.contains(action) {
                return policy.level.clone();
            }
        }
        SecurityLevel::L4Free
    }

    /// 判断动作是否允许执行，不允许时返回错误信息。
    pub fn is_allowed(&self, action: &str, params: &Value) -> Result<(), String> {
        let level = self.check(action, params);
        match level {
            SecurityLevel::L1HardBlocked => {
                if self.block_enabled {
                    Err(format!(
                        "[SECURITY BLOCKED] Action '{}' is hard-blocked by security policy",
                        action
                    ))
                } else {
                    eprintln!(
                        "[SECURITY] Action '{}' is hard-blocked (bypass enabled)",
                        action
                    );
                    Ok(())
                }
            }
            SecurityLevel::L2NeedApproval => {
                if self.block_enabled {
                    Err(format!(
                        "[SECURITY APPROVAL REQUIRED] Action '{}' requires user approval",
                        action
                    ))
                } else {
                    eprintln!(
                        "[SECURITY] Action '{}' requires approval (bypass enabled)",
                        action
                    );
                    Ok(())
                }
            }
            SecurityLevel::L3NeedNotify => {
                if self.notify_enabled {
                    eprintln!(
                        "[SECURITY NOTIFY] Action '{}' executed with notification",
                        action
                    );
                }
                Ok(())
            }
            SecurityLevel::L4Free => Ok(()),
        }
    }

    /// 按名称查找安全策略。
    pub fn get_policy(&self, name: &str) -> Option<&SecurityPolicy> {
        self.policies.iter().find(|p| p.name == name)
    }

    /// 列出所有安全策略。
    pub fn list_policies(&self) -> &[SecurityPolicy] {
        &self.policies
    }

    /// 添加自定义安全策略。
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// 启用/禁用硬拦截（L1HardBlocked + L2NeedApproval）。
    pub fn set_block_enabled(&mut self, enabled: bool) {
        self.block_enabled = enabled;
    }

    /// 启用/禁用通知（L3NeedNotify）。
    pub fn set_notify_enabled(&mut self, enabled: bool) {
        self.notify_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_security_block() {
        let guard = SecurityGuard::new();
        assert!(guard.is_allowed("format_disk", &json!({})).is_err());
        assert!(guard.is_allowed("delete_system_file", &json!({})).is_err());
        assert!(guard
            .is_allowed("modify_system_registry", &json!({}))
            .is_err());
    }

    #[test]
    fn test_security_approval() {
        let guard = SecurityGuard::new();
        assert!(guard.is_allowed("execute_shell", &json!({})).is_err());
        assert!(guard
            .is_allowed("write_outside_workspace", &json!({}))
            .is_err());
    }

    #[test]
    fn test_security_notify() {
        let guard = SecurityGuard::new();
        assert!(guard
            .is_allowed("read_outside_workspace", &json!({}))
            .is_ok());
    }

    #[test]
    fn test_security_free() {
        let guard = SecurityGuard::new();
        assert!(guard.is_allowed("chat", &json!({})).is_ok());
        assert!(guard.is_allowed("search", &json!({})).is_ok());
    }

    #[test]
    fn test_security_bypass() {
        let mut guard = SecurityGuard::new();
        guard.set_block_enabled(false);
        assert!(guard.is_allowed("format_disk", &json!({})).is_ok());
    }

    #[test]
    fn test_check_level() {
        let guard = SecurityGuard::new();
        assert_eq!(
            guard.check("format_disk", &json!({})),
            SecurityLevel::L1HardBlocked
        );
        assert_eq!(
            guard.check("execute_shell", &json!({})),
            SecurityLevel::L2NeedApproval
        );
        assert_eq!(guard.check("chat", &json!({})), SecurityLevel::L4Free);
    }
}
