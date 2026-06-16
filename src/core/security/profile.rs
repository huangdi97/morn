//! Agent security profile types — per-agent sandbox level, permissions, and approval rules.

use crate::core::error::MornError;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityProfile {
    pub agent_id: String,
    pub sandbox_level: u8,
    pub permissions: Vec<String>,
    pub approval_rules: Vec<String>,
}

impl Default for SecurityProfile {
    fn default() -> Self {
        SecurityProfile {
            agent_id: "default".to_string(),
            sandbox_level: 2,
            permissions: vec!["read".to_string(), "chat".to_string()],
            approval_rules: vec!["auto_approve_low_risk".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = SecurityProfile::default();
        assert_eq!(profile.agent_id, "default");
        assert_eq!(profile.sandbox_level, 2);
        assert!(profile.permissions.contains(&"read".to_string()));
    }
}
