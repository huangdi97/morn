//! registry — Stores agent capability registrations and lookup indexes.
use std::collections::HashMap;

use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE, EVENT_SYSTEM_READY};
use crate::core::storage::Storage;

mod capability;
mod version;
mod visibility;

pub use capability::Capability;
pub use version::{compare_versions, AgentTemplate};

#[derive(Clone)]
pub struct Registry {
    capabilities: HashMap<String, Capability>,
    templates: HashMap<String, AgentTemplate>,
    _storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl Registry {
    /// Creates a registry with default capabilities and optional storage and event bus integrations.
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        let mut registry = Registry {
            capabilities: HashMap::new(),
            templates: HashMap::new(),
            _storage: storage,
            event_bus,
        };

        registry.register_defaults();

        if let Some(ref bus) = registry.event_bus {
            bus.publish_event(
                EVENT_SYSTEM_READY,
                "registry",
                serde_json::json!({"status": "ready"}),
            );
        }

        registry
    }

    fn register_defaults(&mut self) {
        let default_cap = Capability::default_chat_agent();
        self.capabilities
            .insert(default_cap.id.clone(), default_cap);

        for template in version::default_templates() {
            self.templates.insert(template.id.clone(), template);
        }
    }

    /// Registers or replaces a capability and emits a registry event when an event bus is configured.
    pub fn register(&mut self, capability: Capability) {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_CHAT_AGENT_RESPONSE,
                "registry",
                serde_json::json!({"action": "register", "capability_id": capability.id}),
            );
        }
        self.capabilities.insert(capability.id.clone(), capability);
    }

    /// Removes a capability by id and returns it when it existed.
    pub fn unregister(&mut self, id: &str) -> Option<Capability> {
        self.capabilities.remove(id)
    }

    /// Finds all capabilities in the given domain and returns references to them.
    pub fn find_by_domain(&self, domain: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.domain == domain)
            .collect()
    }

    /// Finds all capabilities that support the given action and returns references to them.
    pub fn find_by_action(&self, action: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.actions.iter().any(|a| a == action))
            .collect()
    }

    /// Returns references to all registered capabilities.
    pub fn list_all(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    /// Lists capabilities visible to an optional user id and team set.
    pub fn list_available(&self, user_id: Option<&str>, user_teams: &[String]) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| visibility::is_capability_visible(c, user_id, user_teams))
            .collect()
    }

    /// Looks up a capability by id and returns a shared reference when found.
    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// Looks up a capability by id and returns a mutable reference when found.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Capability> {
        self.capabilities.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cap(id: &str, version: &str, domain: &str, actions: Vec<&str>) -> Capability {
        Capability {
            id: id.into(),
            version: version.into(),
            name: id.into(),
            domain: domain.into(),
            actions: actions.into_iter().map(|a| a.into()).collect(),
            description: "test capability".into(),
            trust_score: 70.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "public".into(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        }
    }

    #[test]
    fn test_register_and_get_capability() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));

        let cap = registry.get("cap-1");
        assert!(cap.is_some());
        assert_eq!(cap.map(|c| c.name.as_str()), Some("cap-1"));
    }

    #[test]
    fn test_unregister_removes_capability() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));

        let removed = registry.unregister("cap-1");
        assert!(removed.is_some());
        assert!(registry.get("cap-1").is_none());
    }

    #[test]
    fn test_find_by_domain_and_action() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "analysis", vec!["analyze"]));
        registry.register(make_cap("cap-2", "0.1.0", "research", vec!["search"]));

        assert_eq!(registry.find_by_domain("analysis").len(), 1);
        assert_eq!(registry.find_by_action("search").len(), 1);
    }

    #[test]
    fn test_list_available_respects_visibility() {
        let mut registry = Registry::new(None, None);
        let mut private_cap = make_cap("private-cap", "0.1.0", "general", vec!["chat"]);
        private_cap.visibility = "private".into();
        private_cap.owner_id = Some("user-1".into());
        let mut team_cap = make_cap("team-cap", "0.1.0", "general", vec!["chat"]);
        team_cap.visibility = "team".into();
        team_cap.team_id = Some("team-1".into());
        registry.register(private_cap);
        registry.register(team_cap);

        let teams = vec!["team-1".to_string()];
        let available = registry.list_available(Some("user-1"), &teams);
        assert!(available.iter().any(|c| c.id == "private-cap"));
        assert!(available.iter().any(|c| c.id == "team-cap"));
    }

    #[test]
    fn test_update_trust_score_tracks_stats() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));

        registry.update_trust_score("cap-1", true, 250.0);
        match registry.get("cap-1") {
            Some(cap) => {
                assert_eq!(cap.total_calls, 1);
                assert_eq!(cap.success_calls, 1);
                assert_eq!(cap.avg_latency_ms, 250.0);
            }
            None => panic!("expected cap-1"),
        }
    }

    #[test]
    fn test_templates_have_versions() {
        let registry = Registry::new(None, None);
        let templates = registry.list_templates();
        assert_eq!(templates.len(), 6);
        assert!(templates.iter().all(|t| t.version == "0.1.0"));
        assert_eq!(
            registry
                .get_template("general-assistant")
                .map(|t| t.version.as_str()),
            Some("0.1.0")
        );
    }

    #[test]
    fn test_version_helpers() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));
        registry.register(make_cap("cap-2", "0.2.0", "general", vec!["search"]));

        assert_eq!(registry.get_version("cap-1"), Some("0.1.0"));
        assert_eq!(registry.get_version("general-assistant"), Some("0.1.0"));
        assert_eq!(registry.list_by_version("0.1.0").len(), 2);
        assert!(registry.check_conflict("cap-1", "0.2.0"));
        assert!(!registry.check_conflict("cap-1", "0.1.0"));
        assert!(!registry.check_conflict("missing", "0.1.0"));
    }
}
