use super::{version, AgentTemplate, Capability};
use crate::core::error::MornError;
use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE, EVENT_SYSTEM_READY};
use crate::core::storage::Storage;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn is_capability_visible(
    capability: &Capability,
    user_id: Option<&str>,
    user_teams: &[String],
) -> bool {
    match capability.visibility.as_str() {
        "public" => true,
        "private" => user_id
            .map(|uid| capability.owner_id.as_deref() == Some(uid))
            .unwrap_or(false),
        "team" => capability
            .team_id
            .as_ref()
            .map(|tid| user_teams.iter().any(|ut| ut == tid))
            .unwrap_or(false),
        _ => true,
    }
}

#[derive(Clone)]
pub struct Registry {
    pub capabilities: HashMap<String, Capability>,
    pub templates: HashMap<String, AgentTemplate>,
    pub(super) version_history: HashMap<String, Vec<(String, i64)>>,
    _storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl Registry {
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        let mut registry = Registry {
            capabilities: HashMap::new(),
            templates: HashMap::new(),
            version_history: HashMap::new(),
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
        self.record_version_history(&default_cap);
        self.capabilities
            .insert(default_cap.id.clone(), default_cap);

        for template in version::default_templates() {
            self.templates.insert(template.id.clone(), template);
        }
    }

    fn record_version_history(&mut self, capability: &Capability) {
        self.version_history
            .entry(capability.id.clone())
            .or_default()
            .push((
                capability.version.clone(),
                chrono::Utc::now().timestamp_millis(),
            ));
    }

    pub fn register(&mut self, capability: Capability) {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_CHAT_AGENT_RESPONSE,
                "registry",
                serde_json::json!({"action": "register", "capability_id": capability.id}),
            );
        }
        self.record_version_history(&capability);
        self.capabilities.insert(capability.id.clone(), capability);
    }

    pub fn register_dynamic(&mut self, capability: Capability) -> Result<(), MornError> {
        if let Some(existing) = self.capabilities.get(&capability.id) {
            return Err(MornError::Internal(format!(
                "capability '{}' already registered with version '{}' and name '{}'",
                existing.id, existing.version, existing.name
            )));
        }

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                "registry.capability.registered",
                "registry",
                serde_json::json!({
                    "capability_id": capability.id,
                    "version": capability.version,
                }),
            );
        }

        self.record_version_history(&capability);
        self.capabilities.insert(capability.id.clone(), capability);
        Ok(())
    }

    pub fn create_skeleton(
        &mut self,
        name: &str,
        component_type: &str,
    ) -> Result<Capability, MornError> {
        let name = name.trim();
        let component_type = component_type.trim().to_lowercase();
        if name.is_empty() {
            return Err(MornError::Internal(
                "component skeleton name cannot be empty".to_string(),
            ));
        }
        if component_type.is_empty() {
            return Err(MornError::Internal(
                "component skeleton type cannot be empty".to_string(),
            ));
        }

        let slug = name
            .to_lowercase()
            .chars()
            .map(|ch| if ch.is_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let slug = if slug.is_empty() {
            "component".to_string()
        } else {
            slug
        };

        let capability = Capability {
            id: format!(
                "{}-{}-{}",
                component_type,
                slug,
                uuid::Uuid::new_v4().simple()
            ),
            version: "0.1.0".to_string(),
            name: name.to_string(),
            domain: component_type.clone(),
            actions: Vec::new(),
            description: format!("Empty {} component skeleton", component_type),
            trust_score: 50.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "private".to_string(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        };

        self.register_dynamic(capability.clone())?;
        Ok(capability)
    }

    pub fn unregister(&mut self, id: &str) -> Option<Capability> {
        let removed = self.capabilities.remove(id);
        if removed.is_some() {
            self.version_history.remove(id);
            if let Some(ref bus) = self.event_bus {
                bus.publish_event(
                    "registry.capability.unregistered",
                    "registry",
                    serde_json::json!({"capability_id": id}),
                );
            }
        }
        removed
    }

    pub fn find_by_domain(&self, domain: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.domain == domain)
            .collect()
    }

    pub fn find_by_action(&self, action: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.actions.iter().any(|a| a == action))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    pub fn list_available(&self, user_id: Option<&str>, user_teams: &[String]) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| is_capability_visible(c, user_id, user_teams))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Capability> {
        self.capabilities.get_mut(id)
    }

    pub fn watch_directory<P: AsRef<Path>>(
        &mut self,
        directory: P,
    ) -> Result<Vec<String>, MornError> {
        let directory = directory.as_ref();
        if !directory.exists() {
            return Err(MornError::Internal(format!(
                "registry directory {:?} does not exist",
                directory
            )));
        }
        if !directory.is_dir() {
            return Err(MornError::Internal(format!(
                "registry path {:?} is not a directory",
                directory
            )));
        }

        let mut loaded = Vec::new();
        let entries = fs::read_dir(directory).map_err(|e| {
            MornError::Internal(format!(
                "Cannot read registry directory {:?}: {}",
                directory, e
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                MornError::Internal(format!("Registry directory entry error: {}", e))
            })?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let content = fs::read_to_string(&path).map_err(|e| {
                MornError::Internal(format!("Cannot read registry file {:?}: {}", path, e))
            })?;
            let capability: Capability = serde_json::from_str(&content).map_err(|e| {
                MornError::Internal(format!("Cannot parse registry file {:?}: {}", path, e))
            })?;
            let id = capability.id.clone();
            self.register(capability);
            loaded.push(id);
        }

        Ok(loaded)
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

    #[test]
    fn test_register_dynamic_creates_new_capability() {
        let mut registry = Registry::new(None, None);

        let result =
            registry.register_dynamic(make_cap("dynamic-cap", "0.1.0", "general", vec!["chat"]));

        assert!(result.is_ok());
        assert!(registry.get("dynamic-cap").is_some());
    }

    #[test]
    fn test_register_dynamic_returns_error_on_duplicate() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));

        let result = registry.register_dynamic(make_cap("cap-1", "0.2.0", "general", vec!["chat"]));

        assert!(result.is_err());
        let err = result.expect_err("expected duplicate registration to fail");
        assert!(err.contains("cap-1"));
        assert!(err.contains("0.1.0"));
    }

    #[test]
    fn test_create_skeleton_registers_empty_component() {
        let mut registry = Registry::new(None, None);

        let skeleton = registry
            .create_skeleton("Draft Researcher", "Agent")
            .unwrap();

        assert_eq!(skeleton.name, "Draft Researcher");
        assert_eq!(skeleton.domain, "agent");
        assert!(skeleton.actions.is_empty());
        assert_eq!(
            registry.get(&skeleton.id).map(|cap| cap.name.as_str()),
            Some("Draft Researcher")
        );
    }

    #[test]
    fn test_version_history_is_recorded_correctly() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));
        registry.register(make_cap("cap-1", "0.2.0", "general", vec!["chat"]));

        assert_eq!(
            registry.get_version_history("cap-1"),
            vec!["0.1.0", "0.2.0"]
        );
    }

    #[test]
    fn test_unregister_removes_version_history() {
        let mut registry = Registry::new(None, None);
        registry.register(make_cap("cap-1", "0.1.0", "general", vec!["chat"]));
        assert_eq!(registry.get_version_history("cap-1"), vec!["0.1.0"]);

        registry.unregister("cap-1");

        assert!(registry.get_version_history("cap-1").is_empty());
    }

    #[test]
    fn test_watch_directory_loads_capability_json() {
        let dir = tempfile::tempdir().unwrap();
        let cap = make_cap("hot-cap", "0.1.0", "automation", vec!["run"]);
        std::fs::write(
            dir.path().join("hot-cap.json"),
            serde_json::to_string(&cap).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.path().join("notes.txt"), "ignored").unwrap();

        let mut registry = Registry::new(None, None);
        let loaded = registry.watch_directory(dir.path()).unwrap();

        assert_eq!(loaded, vec!["hot-cap"]);
        assert_eq!(
            registry.get("hot-cap").map(|c| c.domain.as_str()),
            Some("automation")
        );
    }

    #[test]
    fn test_watch_directory_reloads_existing_capability() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("hot-cap.json"),
            serde_json::to_string(&make_cap("hot-cap", "0.1.0", "automation", vec!["run"]))
                .unwrap(),
        )
        .unwrap();

        let mut registry = Registry::new(None, None);
        registry.watch_directory(dir.path()).unwrap();

        std::fs::write(
            dir.path().join("hot-cap.json"),
            serde_json::to_string(&make_cap("hot-cap", "0.2.0", "research", vec!["search"]))
                .unwrap(),
        )
        .unwrap();
        let loaded = registry.watch_directory(dir.path()).unwrap();

        assert_eq!(loaded, vec!["hot-cap"]);
        assert_eq!(registry.get_version("hot-cap"), Some("0.2.0"));
        assert_eq!(registry.find_by_action("search").len(), 1);
        assert_eq!(
            registry.get_version_history("hot-cap"),
            vec!["0.1.0", "0.2.0"]
        );
    }
}
