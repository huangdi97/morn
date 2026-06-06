use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

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

    pub fn install(&mut self, template: WorkflowTemplateEntry) -> Result<(), String> {
        if self.templates.contains_key(&template.workflow_id) {
            return Err(format!(
                "Template '{}' already registered",
                template.workflow_id
            ));
        }
        self.register(template);
        Ok(())
    }

    pub fn from_json_file(path: &Path) -> Result<WorkflowTemplateEntry, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))
    }

    pub fn from_yaml_file(path: &Path) -> Result<WorkflowTemplateEntry, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&content).map_err(|e| format!("YAML parse error: {}", e))
    }

    pub fn from_json(path: &str) -> Result<WorkflowTemplateEntry, String> {
        Self::from_json_file(Path::new(path))
    }

    pub fn from_yaml(path: &str) -> Result<WorkflowTemplateEntry, String> {
        Self::from_yaml_file(Path::new(path))
    }

    pub fn load_json_to_store(&mut self, path: &Path) -> Result<(), String> {
        let template = Self::from_json_file(path)?;
        self.install(template)
    }

    pub fn load_yaml_to_store(&mut self, path: &Path) -> Result<(), String> {
        let template = Self::from_yaml_file(path)?;
        self.install(template)
    }

    pub fn unregister(&mut self, workflow_id: &str) -> Option<WorkflowTemplateEntry> {
        self.templates.remove(workflow_id)
    }

    pub fn len(&self) -> usize {
        self.templates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    fn builtin_templates() -> Vec<WorkflowTemplateEntry> {
        vec![
            WorkflowTemplateEntry {
                workflow_id: "data-analysis".into(),
                name: "数据分析".into(),
                description: "数据导入、清洗、分析与可视化的一站式工作流".into(),
                category: "data".into(),
                nodes: serde_json::json!([
                    {"id": "load_data", "type": "tool", "tool": "data_loader", "params": {"source": ""}},
                    {"id": "clean_data", "type": "tool", "tool": "data_cleaner", "params": {}, "depends_on": ["load_data"]},
                    {"id": "analyze", "type": "llm", "prompt": "对清洗后的数据进行统计分析", "model": "default", "depends_on": ["clean_data"]},
                    {"id": "visualize", "type": "tool", "tool": "chart", "params": {"type": "auto"}, "depends_on": ["analyze"]},
                    {"id": "report", "type": "llm", "prompt": "生成数据分析报告", "model": "default", "depends_on": ["visualize"]}
                ]),
                tags: vec!["data".into(), "analysis".into(), "visualization".into()],
            },
            WorkflowTemplateEntry {
                workflow_id: "code-review".into(),
                name: "代码审查".into(),
                description: "自动代码审查工作流，包括静态分析、安全检查与优化建议".into(),
                category: "development".into(),
                nodes: serde_json::json!([
                    {"id": "fetch_code", "type": "tool", "tool": "git_clone", "params": {"repo": ""}},
                    {"id": "static_analysis", "type": "tool", "tool": "linter", "params": {"rules": "default"}, "depends_on": ["fetch_code"]},
                    {"id": "security_scan", "type": "tool", "tool": "security_checker", "params": {}, "depends_on": ["fetch_code"]},
                    {"id": "llm_review", "type": "llm", "prompt": "审查代码质量、可维护性与最佳实践", "model": "default", "depends_on": ["static_analysis", "security_scan"]},
                    {"id": "summary", "type": "llm", "prompt": "生成代码审查摘要与修改建议", "model": "default", "depends_on": ["llm_review"]}
                ]),
                tags: vec!["code".into(), "review".into(), "security".into()],
            },
            WorkflowTemplateEntry {
                workflow_id: "report-generation".into(),
                name: "报告生成".into(),
                description: "自动收集数据并生成格式化报告".into(),
                category: "reporting".into(),
                nodes: serde_json::json!([
                    {"id": "collect", "type": "tool", "tool": "web_search", "params": {"query": ""}},
                    {"id": "research", "type": "agent", "agent_id": "researcher", "input": "", "depends_on": ["collect"]},
                    {"id": "draft", "type": "llm", "prompt": "撰写包含摘要、发现、分析与建议的完整报告", "model": "default", "depends_on": ["research"]},
                    {"id": "format", "type": "tool", "tool": "formatter", "params": {"style": "markdown"}, "depends_on": ["draft"]},
                    {"id": "deliver", "type": "notification", "channel": "email", "message": "报告已生成", "depends_on": ["format"]}
                ]),
                tags: vec!["report".into(), "generate".into(), "document".into()],
            },
            WorkflowTemplateEntry {
                workflow_id: "web-scraping".into(),
                name: "网页抓取".into(),
                description: "网页内容抓取、解析与结构化存储工作流".into(),
                category: "data".into(),
                nodes: serde_json::json!([
                    {"id": "fetch_page", "type": "tool", "tool": "http_request", "params": {"url": "", "method": "GET"}},
                    {"id": "parse_html", "type": "tool", "tool": "html_parser", "params": {"selector": ""}, "depends_on": ["fetch_page"]},
                    {"id": "extract_data", "type": "llm", "prompt": "从解析后的HTML中提取结构化数据", "model": "default", "depends_on": ["parse_html"]},
                    {"id": "transform", "type": "tool", "tool": "data_transformer", "params": {"format": "json"}, "depends_on": ["extract_data"]},
                    {"id": "save", "type": "tool", "tool": "file_writer", "params": {"path": ""}, "depends_on": ["transform"]}
                ]),
                tags: vec!["web".into(), "scraping".into(), "crawl".into()],
            },
            WorkflowTemplateEntry {
                workflow_id: "scheduled-monitoring".into(),
                name: "定时监控".into(),
                description: "定时检查系统健康状态并在异常时发送告警".into(),
                category: "operations".into(),
                nodes: serde_json::json!([
                    {"id": "health_check", "type": "tool", "tool": "http_request", "params": {"url": "", "method": "GET", "timeout": 10}},
                    {"id": "check_metrics", "type": "tool", "tool": "metric_collector", "params": {}, "depends_on": ["health_check"]},
                    {"id": "evaluate", "type": "llm", "prompt": "根据收集到的指标评估系统健康状态", "model": "default", "depends_on": ["check_metrics"]},
                    {"id": "conditional_alert", "type": "condition", "expression": "status != healthy", "true_branch": [{"id": "alert", "type": "notification", "channel": "default", "message": "系统异常：请立即检查"}], "false_branch": [], "depends_on": ["evaluate"]},
                    {"id": "log_result", "type": "tool", "tool": "logger", "params": {"level": "info"}, "depends_on": ["conditional_alert"]}
                ]),
                tags: vec!["monitor".into(), "scheduled".into(), "ops".into()],
            },
        ]
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
