//! community_templates — Manages community-provided workflow template registries.
use std::collections::HashMap;

mod fetch;
mod store;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub download_url: String,
    pub checksum: Option<String>,
    pub min_engine_version: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryInfo {
    pub name: String,
    pub url: String,
    pub description: String,
    pub template_count: usize,
}

pub struct CommunityTemplateRegistry {
    registry_url: String,
    cache: HashMap<String, RemoteTemplate>,
    registries: Vec<RegistryInfo>,
    installed: HashMap<String, RemoteTemplate>,
}

impl CommunityTemplateRegistry {
    pub fn new(registry_url: &str) -> Self {
        CommunityTemplateRegistry {
            registry_url: registry_url.to_string(),
            cache: HashMap::new(),
            registries: Vec::new(),
            installed: HashMap::new(),
        }
    }

    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }

    pub fn set_registry_url(&mut self, url: String) {
        self.registry_url = url;
    }

    pub fn installed_count(&self) -> usize {
        self.installed.len()
    }

    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }
}

impl Default for CommunityTemplateRegistry {
    fn default() -> Self {
        CommunityTemplateRegistry::new("https://templates.morn.ai/community")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = CommunityTemplateRegistry::new("https://example.com/registry");
        assert_eq!(registry.installed_count(), 0);
        assert_eq!(registry.cached_count(), 0);
    }

    #[test]
    fn test_install_templates_no_cache() {
        let mut registry = CommunityTemplateRegistry::new("https://0.0.0.0/nonexistent");
        let result = registry.install_templates(&["test-flow".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_install_templates_already_installed() {
        let mut registry = CommunityTemplateRegistry::new("https://example.com/registry");
        let template = RemoteTemplate {
            id: "dup".into(),
            name: "Duplicate".into(),
            description: "Test".into(),
            author: Some("test".into()),
            version: "1.0.0".into(),
            category: "general".into(),
            tags: vec![],
            download_url: "https://example.com/dup.zip".into(),
            checksum: None,
            min_engine_version: None,
            updated_at: None,
        };
        registry.cache.insert("dup".into(), template.clone());
        registry.installed.insert("dup".into(), template);

        let result = registry.install_templates(&["dup".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_install_templates_not_found() {
        let mut registry = CommunityTemplateRegistry::new("https://example.com/registry");
        let template = RemoteTemplate {
            id: "existing".into(),
            name: "Existing".into(),
            description: "Test".into(),
            author: None,
            version: "1.0.0".into(),
            category: "general".into(),
            tags: vec![],
            download_url: "https://example.com/existing.zip".into(),
            checksum: None,
            min_engine_version: None,
            updated_at: None,
        };
        registry.cache.insert("existing".into(), template);

        let result = registry.install_templates(&["missing".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let mut registry = CommunityTemplateRegistry::default();
        let result = registry.uninstall("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_installed() {
        let mut registry = CommunityTemplateRegistry::default();
        let template = RemoteTemplate {
            id: "test-flow".into(),
            name: "Test Flow".into(),
            description: "A test template".into(),
            author: Some("author".into()),
            version: "1.0.0".into(),
            category: "general".into(),
            tags: vec!["test".into()],
            download_url: "https://example.com/test.zip".into(),
            checksum: None,
            min_engine_version: None,
            updated_at: None,
        };
        registry.cache.insert("test-flow".into(), template.clone());
        registry.installed.insert("test-flow".into(), template);

        assert!(registry.is_installed("test-flow"));
        assert!(!registry.is_installed("missing"));
    }

    #[test]
    fn test_list_installed_sorted() {
        let mut registry = CommunityTemplateRegistry::default();
        let a = RemoteTemplate {
            id: "b-flow".into(),
            name: "B Flow".into(),
            description: "".into(),
            author: None,
            version: "1.0.0".into(),
            category: "general".into(),
            tags: vec![],
            download_url: "".into(),
            checksum: None,
            min_engine_version: None,
            updated_at: None,
        };
        let b = RemoteTemplate {
            id: "a-flow".into(),
            name: "A Flow".into(),
            description: "".into(),
            author: None,
            version: "1.0.0".into(),
            category: "general".into(),
            tags: vec![],
            download_url: "".into(),
            checksum: None,
            min_engine_version: None,
            updated_at: None,
        };
        registry.installed.insert("b-flow".into(), a);
        registry.installed.insert("a-flow".into(), b);

        let list = registry.list_installed();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "a-flow");
        assert_eq!(list[1].id, "b-flow");
    }

    #[test]
    fn test_search_by_name() {
        let mut registry = CommunityTemplateRegistry::default();
        registry.cache.insert(
            "data-pipeline".into(),
            RemoteTemplate {
                id: "data-pipeline".into(),
                name: "Data Pipeline".into(),
                description: "ETL pipeline template".into(),
                author: None,
                version: "1.0.0".into(),
                category: "data".into(),
                tags: vec!["etl".into(), "pipeline".into()],
                download_url: "".into(),
                checksum: None,
                min_engine_version: None,
                updated_at: None,
            },
        );
        registry.cache.insert(
            "web-app".into(),
            RemoteTemplate {
                id: "web-app".into(),
                name: "Web Application".into(),
                description: "Full stack web app".into(),
                author: None,
                version: "1.0.0".into(),
                category: "web".into(),
                tags: vec!["react".into(), "node".into()],
                download_url: "".into(),
                checksum: None,
                min_engine_version: None,
                updated_at: None,
            },
        );

        let results = registry.search("data");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "data-pipeline");
    }

    #[test]
    fn test_search_by_tag() {
        let mut registry = CommunityTemplateRegistry::default();
        registry.cache.insert(
            "tagged-flow".into(),
            RemoteTemplate {
                id: "tagged-flow".into(),
                name: "Tagged Flow".into(),
                description: "".into(),
                author: None,
                version: "1.0.0".into(),
                category: "general".into(),
                tags: vec!["featured".into(), "important".into()],
                download_url: "".into(),
                checksum: None,
                min_engine_version: None,
                updated_at: None,
            },
        );

        let results = registry.search("featured");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_results() {
        let registry = CommunityTemplateRegistry::default();
        let results = registry.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_add_registry() {
        let mut registry = CommunityTemplateRegistry::default();
        let info = RegistryInfo {
            name: "Community".into(),
            url: "https://community.example.com".into(),
            description: "Community registry".into(),
            template_count: 10,
        };
        registry.add_registry(info);
        assert_eq!(registry.list_registries().len(), 1);
    }

    #[test]
    fn test_clear_cache() {
        let mut registry = CommunityTemplateRegistry::default();
        registry.cache.insert(
            "temp".into(),
            RemoteTemplate {
                id: "temp".into(),
                name: "Temp".into(),
                description: "".into(),
                author: None,
                version: "1.0.0".into(),
                category: "general".into(),
                tags: vec![],
                download_url: "".into(),
                checksum: None,
                min_engine_version: None,
                updated_at: None,
            },
        );
        assert_eq!(registry.cached_count(), 1);
        registry.clear_cache();
        assert_eq!(registry.cached_count(), 0);
    }

    #[test]
    fn test_set_registry_url() {
        let mut registry = CommunityTemplateRegistry::new("https://old.url");
        assert_eq!(registry.registry_url(), "https://old.url");
        registry.set_registry_url("https://new.url".into());
        assert_eq!(registry.registry_url(), "https://new.url");
    }

    #[test]
    fn test_remote_template_serialization() {
        let template = RemoteTemplate {
            id: "serialize-test".into(),
            name: "Serialize Test".into(),
            description: "Testing serialization".into(),
            author: Some("tester".into()),
            version: "1.2.3".into(),
            category: "test".into(),
            tags: vec!["test".into()],
            download_url: "https://example.com/test.zip".into(),
            checksum: Some("abc123".into()),
            min_engine_version: Some("1.0.0".into()),
            updated_at: Some("2025-01-01".into()),
        };
        let json = serde_json::to_string(&template).unwrap();
        let deserialized: RemoteTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "serialize-test");
        assert_eq!(deserialized.version, "1.2.3");
        assert_eq!(deserialized.checksum, Some("abc123".into()));
    }

    #[test]
    fn test_check_updates_no_installed() {
        let mut registry = CommunityTemplateRegistry::new("https://0.0.0.0/nonexistent");
        let result = registry.check_updates(None);
        assert!(result.is_err());
    }
}
