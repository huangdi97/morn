//! template_store — Manages workflow template manifests across local and remote stores.
use std::collections::HashMap;

mod local;
mod remote;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateVersion {
    pub version: String,
    pub changelog: Option<String>,
    pub min_engine_version: Option<String>,
    pub compatible: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub current_version: String,
    pub versions: Vec<TemplateVersion>,
    pub category: String,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateStore {
    templates: HashMap<String, TemplateManifest>,
}

impl Default for TemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateStore {
    pub fn new() -> Self {
        TemplateStore {
            templates: HashMap::new(),
        }
    }
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u64> = a.split('.').filter_map(|s| s.parse::<u64>().ok()).collect();
    let b_parts: Vec<u64> = b.split('.').filter_map(|s| s.parse::<u64>().ok()).collect();

    for i in 0..a_parts.len().max(b_parts.len()) {
        let a_v = a_parts.get(i).copied().unwrap_or(0);
        let b_v = b_parts.get(i).copied().unwrap_or(0);
        match a_v.cmp(&b_v) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(id: &str, version: &str) -> TemplateManifest {
        TemplateManifest {
            id: id.to_string(),
            name: format!("Template {}", id),
            description: format!("Description for {}", id),
            author: Some("test".into()),
            current_version: version.to_string(),
            versions: vec![TemplateVersion {
                version: version.to_string(),
                changelog: None,
                min_engine_version: None,
                compatible: true,
            }],
            category: "general".into(),
            tags: vec![],
            source_url: None,
            registry_url: None,
        }
    }

    #[test]
    fn test_new_store_is_empty() {
        let store = TemplateStore::new();
        assert!(store.is_empty());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_install_template() {
        let mut store = TemplateStore::new();
        let m = sample_manifest("test-flow", "1.0.0");
        assert!(store.install(m).is_ok());
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_install_duplicate_fails() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("dup", "1.0.0")).unwrap();
        let result = store.install(sample_manifest("dup", "2.0.0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_uninstall_template() {
        let mut store = TemplateStore::new();
        store
            .install(sample_manifest("to-remove", "1.0.0"))
            .unwrap();
        let removed = store.uninstall("to-remove");
        assert!(removed.is_ok());
        assert!(store.is_empty());
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let mut store = TemplateStore::new();
        let result = store.uninstall("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_template() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("my-flow", "1.0.0")).unwrap();
        let t = store.get("my-flow");
        assert!(t.is_some());
        assert_eq!(t.unwrap().current_version, "1.0.0");
    }

    #[test]
    fn test_get_nonexistent() {
        let store = TemplateStore::new();
        assert!(store.get("nope").is_none());
    }

    #[test]
    fn test_list_templates() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("b", "1.0.0")).unwrap();
        store.install(sample_manifest("a", "1.0.0")).unwrap();
        let list = store.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "a");
        assert_eq!(list[1].id, "b");
    }

    #[test]
    fn test_list_by_category() {
        let mut store = TemplateStore::new();
        let mut m = sample_manifest("data-flow", "1.0.0");
        m.category = "data".into();
        store.install(m).unwrap();
        store.install(sample_manifest("gen-flow", "1.0.0")).unwrap();

        assert_eq!(store.list_by_category("data").len(), 1);
        assert_eq!(store.list_by_category("general").len(), 1);
        assert_eq!(store.list_by_category("nonexistent").len(), 0);
    }

    #[test]
    fn test_list_by_tag() {
        let mut store = TemplateStore::new();
        let mut m = sample_manifest("tagged-flow", "1.0.0");
        m.tags = vec!["important".into(), "featured".into()];
        store.install(m).unwrap();
        store
            .install(sample_manifest("plain-flow", "1.0.0"))
            .unwrap();

        assert_eq!(store.list_by_tag("important").len(), 1);
        assert_eq!(store.list_by_tag("nonexistent").len(), 0);
    }

    #[test]
    fn test_is_installed() {
        let mut store = TemplateStore::new();
        store
            .install(sample_manifest("installed", "1.0.0"))
            .unwrap();
        assert!(store.is_installed("installed"));
        assert!(!store.is_installed("missing"));
    }

    #[test]
    fn test_get_version() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("ver-test", "2.5.0")).unwrap();
        assert_eq!(store.get_version("ver-test"), Some("2.5.0"));
        assert_eq!(store.get_version("missing"), None);
    }

    #[test]
    fn test_update_existing() {
        let mut store = TemplateStore::new();
        store
            .install(sample_manifest("updatable", "1.0.0"))
            .unwrap();

        let mut updated = sample_manifest("updatable", "2.0.0");
        updated.description = "Updated description".into();
        assert!(store.update(updated).is_ok());
        assert_eq!(store.get_version("updatable"), Some("2.0.0"));
    }

    #[test]
    fn test_update_nonexistent_fails() {
        let mut store = TemplateStore::new();
        let result = store.update(sample_manifest("missing", "1.0.0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions_equal() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_compare_versions_less() {
        assert_eq!(compare_versions("1.0.0", "2.0.0"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_versions_greater() {
        assert_eq!(
            compare_versions("3.0.0", "2.0.0"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_compare_versions_different_length() {
        assert_eq!(compare_versions("1.0", "1.0.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_compare_versions_patch() {
        assert_eq!(compare_versions("1.0.1", "1.0.2"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_has_update_true() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("check", "1.0.0")).unwrap();
        assert!(store.has_update("check", "2.0.0").unwrap());
    }

    #[test]
    fn test_has_update_false() {
        let mut store = TemplateStore::new();
        store.install(sample_manifest("check", "2.0.0")).unwrap();
        assert!(!store.has_update("check", "1.0.0").unwrap());
    }

    #[test]
    fn test_has_update_nonexistent() {
        let store = TemplateStore::new();
        let result = store.has_update("missing", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let m = sample_manifest("serialize-test", "1.0.0");
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: TemplateManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "serialize-test");
        assert_eq!(deserialized.current_version, "1.0.0");
    }

    #[test]
    fn test_template_version_defaults() {
        let v = TemplateVersion {
            version: "1.0.0".into(),
            changelog: None,
            min_engine_version: None,
            compatible: true,
        };
        assert!(v.compatible);
    }

    #[test]
    fn test_bulk_install_empty_registry() {
        let mut store = TemplateStore::new();
        let result = store.bulk_install_from_registry("https://example.com/nonexistent-registry");
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_remote_registry_bad_url() {
        let result = TemplateStore::fetch_remote_registry("https://0.0.0.0/nonexistent");
        assert!(result.is_err());
    }
}
