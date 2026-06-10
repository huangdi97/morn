//! Security guard — policy enforcement, action checking, and profile management.

use serde_json::Value;
use tracing;

use super::constitution::{SecurityLevel, SecurityPolicy};
use super::profile::SecurityProfile;

pub struct SecurityGuard {
    policies: Vec<SecurityPolicy>,
    pub block_enabled: bool,
    pub notify_enabled: bool,
    profiles: Vec<SecurityProfile>,
}

impl SecurityGuard {
    /// Creates a security guard with the default policy set enabled.
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
                name: "access_other_process_memory".to_string(),
                level: SecurityLevel::L1HardBlocked,
                pattern: "access_other_process_memory".to_string(),
                description: "Access other process memory".to_string(),
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
                name: "sandbox_code_execution".to_string(),
                level: SecurityLevel::L2NeedApproval,
                pattern: "sandbox_code_execution".to_string(),
                description: "Execute code in sandbox".to_string(),
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
            profiles: vec![SecurityProfile::default()],
        }
    }

    /// Checks an action and parameters against policies, returning the required security level.
    pub fn check(&self, action: &str, _params: &Value) -> SecurityLevel {
        for policy in &self.policies {
            if action.contains(&policy.pattern) || policy.pattern.contains(action) {
                return policy.level.clone();
            }
        }
        SecurityLevel::L4Free
    }

    /// Validates whether an action is allowed and returns an error for blocked or approval-required actions.
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
                    tracing::info!(
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
                    tracing::info!(
                        "[SECURITY] Action '{}' requires approval (bypass enabled)",
                        action
                    );
                    Ok(())
                }
            }
            SecurityLevel::L3NeedNotify => {
                if self.notify_enabled {
                    tracing::info!(
                        "[SECURITY NOTIFY] Action '{}' executed with notification",
                        action
                    );
                }
                Ok(())
            }
            SecurityLevel::L4Free => Ok(()),
        }
    }

    /// Looks up a security policy by name and returns it when found.
    pub fn get_policy(&self, name: &str) -> Option<&SecurityPolicy> {
        self.policies.iter().find(|p| p.name == name)
    }

    /// Returns the configured security policy list.
    pub fn list_policies(&self) -> &[SecurityPolicy] {
        &self.policies
    }

    /// Adds a security policy to the active policy list.
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Enables or disables blocking for hard-blocked and approval-required actions.
    pub fn set_block_enabled(&mut self, enabled: bool) {
        self.block_enabled = enabled;
    }

    /// Enables or disables notifications for notify-level actions.
    pub fn set_notify_enabled(&mut self, enabled: bool) {
        self.notify_enabled = enabled;
    }

    pub fn get_profile(&self, agent_id: &str) -> Option<&SecurityProfile> {
        self.profiles.iter().find(|p| p.agent_id == agent_id)
    }

    pub fn set_profile(&mut self, profile: SecurityProfile) {
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|p| p.agent_id == profile.agent_id)
        {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn list_profiles(&self) -> &[SecurityProfile] {
        &self.profiles
    }
}

impl Default for SecurityGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::constitution::SecurityLevel;
    use super::super::profile::SecurityProfile;
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

    #[test]
    fn test_access_other_process_memory_blocked() {
        let guard = SecurityGuard::new();
        assert!(guard
            .is_allowed("access_other_process_memory", &json!({}))
            .is_err());
    }

    #[test]
    fn test_sandbox_code_execution_requires_approval() {
        let guard = SecurityGuard::new();
        assert!(guard
            .is_allowed("sandbox_code_execution", &json!({}))
            .is_err());
    }

    #[test]
    fn test_set_and_get_profile() {
        let mut guard = SecurityGuard::new();
        let profile = SecurityProfile {
            agent_id: "agent-custom".to_string(),
            sandbox_level: 3,
            permissions: vec!["execute".to_string()],
            approval_rules: vec!["require_ceo".to_string()],
        };
        guard.set_profile(profile);
        let retrieved = guard.get_profile("agent-custom");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sandbox_level, 3);
    }

    #[test]
    fn test_override_existing_profile() {
        let mut guard = SecurityGuard::new();
        let profile = SecurityProfile {
            agent_id: "default".to_string(),
            sandbox_level: 1,
            permissions: vec![],
            approval_rules: vec![],
        };
        guard.set_profile(profile);
        let retrieved = guard.get_profile("default").unwrap();
        assert_eq!(retrieved.sandbox_level, 1);
    }
}
