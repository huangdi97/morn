//! network_permissions — Runtime network access grants for components.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAccessLevel {
    L0NoNetwork,
    L1SpecifiedDomains(Vec<String>),
    L2FullNetwork,
}

pub struct PermissionManager {
    grants: HashMap<String, NetworkAccessLevel>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
        }
    }

    pub fn set_permission(&mut self, component_id: &str, level: NetworkAccessLevel) {
        self.grants.insert(component_id.to_string(), level);
    }

    pub fn get_permission(&self, component_id: &str) -> Option<NetworkAccessLevel> {
        self.grants.get(component_id).cloned()
    }

    pub fn check_access(&self, component_id: &str, domain: &str) -> bool {
        match self.grants.get(component_id) {
            Some(NetworkAccessLevel::L0NoNetwork) | None => false,
            Some(NetworkAccessLevel::L2FullNetwork) => true,
            Some(NetworkAccessLevel::L1SpecifiedDomains(allowed)) => allowed
                .iter()
                .any(|allowed_domain| domain_matches(allowed_domain, domain)),
        }
    }

    pub fn revoke(&mut self, component_id: &str) {
        self.grants.remove(component_id);
    }

    pub fn list_grants(&self) -> HashMap<String, NetworkAccessLevel> {
        self.grants.clone()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn domain_matches(allowed_domain: &str, requested_domain: &str) -> bool {
    let allowed = allowed_domain.trim().trim_end_matches('.').to_lowercase();
    let requested = requested_domain.trim().trim_end_matches('.').to_lowercase();

    if let Some(suffix) = allowed.strip_prefix("*.") {
        requested == suffix || requested.ends_with(&format!(".{}", suffix))
    } else {
        requested == allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_and_no_network_access() {
        let mut manager = PermissionManager::new();
        manager.set_permission("agent-a", NetworkAccessLevel::L2FullNetwork);
        manager.set_permission("agent-b", NetworkAccessLevel::L0NoNetwork);

        assert!(manager.check_access("agent-a", "example.com"));
        assert!(!manager.check_access("agent-b", "example.com"));
        assert!(!manager.check_access("unknown", "example.com"));
    }

    #[test]
    fn test_specified_domains_and_revoke() {
        let mut manager = PermissionManager::new();
        manager.set_permission(
            "tool",
            NetworkAccessLevel::L1SpecifiedDomains(vec![
                "api.example.com".to_string(),
                "*.local.test".to_string(),
            ]),
        );

        assert!(manager.check_access("tool", "api.example.com"));
        assert!(manager.check_access("tool", "sub.local.test"));
        assert!(!manager.check_access("tool", "other.example.com"));

        manager.revoke("tool");
        assert_eq!(manager.get_permission("tool"), None);
        assert!(manager.list_grants().is_empty());
    }
}
