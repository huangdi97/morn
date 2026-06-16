//! computer — Exposes controlled computer operations and perception helpers.
pub mod app_ops;
pub mod browser_ops;
pub mod desktop_ops;
pub mod fs_ops;
pub mod perception;
pub mod sys_ops;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SecurityLevel {
    L1Sandbox,
    L2Local,
    L3System,
}

impl SecurityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityLevel::L1Sandbox => "sandbox",
            SecurityLevel::L2Local => "local",
            SecurityLevel::L3System => "system",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityConfig {
    pub default_level: SecurityLevel,
    #[serde(default)]
    pub per_agent_levels: HashMap<String, SecurityLevel>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            default_level: SecurityLevel::L1Sandbox,
            per_agent_levels: HashMap::new(),
        }
    }
}

impl SecurityConfig {
    pub fn new(default_level: SecurityLevel) -> Self {
        SecurityConfig {
            default_level,
            per_agent_levels: HashMap::new(),
        }
    }

    pub fn level_for_agent(&self, agent_id: &str) -> &SecurityLevel {
        self.per_agent_levels
            .get(agent_id)
            .unwrap_or(&self.default_level)
    }

    pub fn set_agent_level(&mut self, agent_id: impl Into<String>, level: SecurityLevel) {
        self.per_agent_levels.insert(agent_id.into(), level);
    }

    pub fn approval_required_for_agent(&self, agent_id: &str) -> bool {
        matches!(self.level_for_agent(agent_id), SecurityLevel::L3System)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputerOpResult {
    pub success: bool,
    pub data: String,
    pub security_level: String,
    pub approval_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_level_strings_are_stable() {
        assert_eq!(SecurityLevel::L1Sandbox.as_str(), "sandbox");
        assert_eq!(SecurityLevel::L2Local.as_str(), "local");
        assert_eq!(SecurityLevel::L3System.as_str(), "system");
    }

    #[test]
    fn security_level_supports_equality() {
        assert_eq!(SecurityLevel::L2Local, SecurityLevel::L2Local);
        assert_ne!(SecurityLevel::L1Sandbox, SecurityLevel::L3System);
    }

    #[test]
    fn computer_op_result_carries_required_fields() {
        let result = ComputerOpResult {
            success: true,
            data: "ok".to_string(),
            security_level: SecurityLevel::L1Sandbox.as_str().to_string(),
            approval_required: false,
        };

        assert!(result.success);
        assert_eq!(result.data, "ok");
        assert_eq!(result.security_level, "sandbox");
        assert!(!result.approval_required);
    }

    #[test]
    fn security_config_supports_per_agent_overrides() {
        let mut config = SecurityConfig::new(SecurityLevel::L1Sandbox);
        config.set_agent_level("desktop-agent", SecurityLevel::L3System);

        assert_eq!(
            config.level_for_agent("desktop-agent"),
            &SecurityLevel::L3System
        );
        assert_eq!(
            config.level_for_agent("unknown-agent"),
            &SecurityLevel::L1Sandbox
        );
        assert!(config.approval_required_for_agent("desktop-agent"));
        assert!(!config.approval_required_for_agent("unknown-agent"));
    }
}
