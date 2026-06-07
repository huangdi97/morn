//! workflow_templates — Manages built-in and custom workflow template catalog entries.
use serde_json::Value;
use std::collections::HashMap;

mod builtins;
mod custom;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTemplateEntry {
    pub workflow_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub nodes: Value,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowTemplateStore {
    templates: HashMap<String, WorkflowTemplateEntry>,
}

impl WorkflowTemplateStore {
    pub fn new() -> Self {
        let mut store = WorkflowTemplateStore {
            templates: HashMap::new(),
        };
        for t in Self::builtin_templates() {
            store.register(t);
        }
        store
    }

    pub fn register(&mut self, template: WorkflowTemplateEntry) {
        self.templates
            .insert(template.workflow_id.clone(), template);
    }

    pub fn get(&self, workflow_id: &str) -> Option<&WorkflowTemplateEntry> {
        self.templates.get(workflow_id)
    }

    pub fn discover(&self) -> Vec<&WorkflowTemplateEntry> {
        let mut list: Vec<_> = self.templates.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn discover_by_category(&self, category: &str) -> Vec<&WorkflowTemplateEntry> {
        let mut list: Vec<_> = self
            .templates
            .values()
            .filter(|t| t.category == category)
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn discover_by_tag(&self, tag: &str) -> Vec<&WorkflowTemplateEntry> {
        let mut list: Vec<_> = self
            .templates
            .values()
            .filter(|t| t.tags.iter().any(|t| t == tag))
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn len(&self) -> usize {
        self.templates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    fn builtin_templates() -> Vec<WorkflowTemplateEntry> {
        builtins::builtin_templates()
    }
}

impl Default for WorkflowTemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_new_store_has_builtins() {
        let store = WorkflowTemplateStore::new();
        assert_eq!(store.len(), 5);
    }

    #[test]
    fn test_get_template() {
        let store = WorkflowTemplateStore::new();
        let t = store.get("data-analysis");
        assert!(t.is_some());
        assert_eq!(t.unwrap().name, "数据分析");
    }

    #[test]
    fn test_get_nonexistent() {
        let store = WorkflowTemplateStore::new();
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn test_discover_all() {
        let store = WorkflowTemplateStore::new();
        let all = store.discover();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_discover_by_category() {
        let store = WorkflowTemplateStore::new();
        let data_templates = store.discover_by_category("data");
        assert_eq!(data_templates.len(), 2);
    }

    #[test]
    fn test_discover_by_tag() {
        let store = WorkflowTemplateStore::new();
        let ops = store.discover_by_tag("ops");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].workflow_id, "scheduled-monitoring");
    }

    #[test]
    fn test_install_new() {
        let mut store = WorkflowTemplateStore::new();
        let t = WorkflowTemplateEntry {
            workflow_id: "custom-flow".into(),
            name: "自定义流程".into(),
            description: "测试".into(),
            category: "general".into(),
            nodes: serde_json::json!([{"id": "step1", "type": "llm", "prompt": "hello"}]),
            tags: vec!["test".into()],
        };
        assert!(store.install(t).is_ok());
        assert_eq!(store.len(), 6);
    }

    #[test]
    fn test_install_duplicate_fails() {
        let mut store = WorkflowTemplateStore::new();
        let t = WorkflowTemplateEntry {
            workflow_id: "data-analysis".into(),
            name: "重名".into(),
            description: "".into(),
            category: "".into(),
            nodes: serde_json::json!([]),
            tags: vec![],
        };
        assert!(store.install(t).is_err());
    }

    #[test]
    fn test_unregister() {
        let mut store = WorkflowTemplateStore::new();
        let removed = store.unregister("data-analysis");
        assert!(removed.is_some());
        assert_eq!(store.len(), 4);
    }

    #[test]
    fn test_from_json_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let content = r#"{
            "workflow_id": "json-test",
            "name": "JSON Test",
            "description": "from json",
            "category": "test",
            "nodes": [{"id": "a", "type": "llm", "prompt": "test"}],
            "tags": ["json"]
        }"#;
        file.write_all(content.as_bytes()).unwrap();
        let t = WorkflowTemplateStore::from_json_file(file.path()).unwrap();
        assert_eq!(t.workflow_id, "json-test");
        assert_eq!(t.tags, vec!["json"]);
    }

    #[test]
    fn test_from_yaml_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let content = r#"
workflow_id: yaml-test
name: YAML Test
description: from yaml
category: test
nodes:
  - id: a
    type: llm
    prompt: test
tags:
  - yaml
"#;
        file.write_all(content.as_bytes()).unwrap();
        let t = WorkflowTemplateStore::from_yaml_file(file.path()).unwrap();
        assert_eq!(t.workflow_id, "yaml-test");
        assert_eq!(t.tags, vec!["yaml"]);
    }

    #[test]
    fn test_load_json_to_store() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let content = r#"{"workflow_id":"load-test","name":"Load Test","description":"","category":"","nodes":[],"tags":[]}"#;
        file.write_all(content.as_bytes()).unwrap();
        let mut store = WorkflowTemplateStore::new();
        store.load_json_to_store(file.path()).unwrap();
        assert_eq!(store.len(), 6);
        assert!(store.get("load-test").is_some());
    }

    #[test]
    fn test_from_json_str() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let content = r#"{"workflow_id":"str-test","name":"Str","description":"","category":"","nodes":[],"tags":[]}"#;
        file.write_all(content.as_bytes()).unwrap();
        let t = WorkflowTemplateStore::from_json(file.path().to_str().unwrap()).unwrap();
        assert_eq!(t.workflow_id, "str-test");
    }

    #[test]
    fn test_from_yaml_str() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let content = "workflow_id: yaml-str\nname: YAML Str\ndescription: test\ncategory: test\nnodes:\n  - id: a\ntags: []\n";
        file.write_all(content.as_bytes()).unwrap();
        let t = WorkflowTemplateStore::from_yaml(file.path().to_str().unwrap()).unwrap();
        assert_eq!(t.workflow_id, "yaml-str");
    }

    #[test]
    fn test_is_empty() {
        let store = WorkflowTemplateStore::new();
        assert!(!store.is_empty());
    }

    #[test]
    fn test_all_templates_have_nodes() {
        for t in WorkflowTemplateStore::builtin_templates() {
            let nodes = t.nodes.as_array().unwrap();
            assert!(
                !nodes.is_empty(),
                "Template '{}' has no nodes",
                t.workflow_id
            );
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let t = WorkflowTemplateEntry {
            workflow_id: "roundtrip".into(),
            name: "Roundtrip".into(),
            description: "test".into(),
            category: "test".into(),
            nodes: serde_json::json!([{"x": 1}]),
            tags: vec!["t1".into(), "t2".into()],
        };
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: WorkflowTemplateEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.workflow_id, "roundtrip");
        assert_eq!(deserialized.tags.len(), 2);
    }

    #[test]
    fn test_from_nonexistent_file() {
        let result = WorkflowTemplateStore::from_json("/nonexistent/file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_by_tag_multiple() {
        let store = WorkflowTemplateStore::new();
        let tagged = store.discover_by_tag("data");
        assert!(tagged.len() >= 1);
    }
}
