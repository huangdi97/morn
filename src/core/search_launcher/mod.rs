//! search_launcher — Provides launcher search across local, cached, and web sources.
use crate::core::error::MornError;
use std::collections::HashMap;

mod cache;
mod local;
mod web;

pub use cache::SearchIndex;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SearchCategory {
    App,
    File,
    Command,
    AgentSkill,
}

impl SearchCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchCategory::App => "app",
            SearchCategory::File => "file",
            SearchCategory::Command => "command",
            SearchCategory::AgentSkill => "agent_skill",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SearchCategory,
    pub keywords: Vec<String>,
    pub score: f64,
    pub metadata: HashMap<String, String>,
}

impl SearchItem {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: SearchCategory,
    ) -> Self {
        SearchItem {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            keywords: Vec::new(),
            score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SearchLauncher {
    index: SearchIndex,
    enabled: bool,
    hotkey: String,
}

impl Default for SearchLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchLauncher {
    pub fn new() -> Self {
        SearchLauncher {
            index: SearchIndex::new(),
            enabled: true,
            hotkey: "Alt+Space".into(),
        }
    }

    pub fn with_hotkey(hotkey: impl Into<String>) -> Self {
        SearchLauncher {
            index: SearchIndex::new(),
            enabled: true,
            hotkey: hotkey.into(),
        }
    }

    pub fn hotkey(&self) -> &str {
        &self.hotkey
    }

    pub fn set_hotkey(&mut self, hotkey: impl Into<String>) {
        self.hotkey = hotkey.into();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn index(&self) -> &SearchIndex {
        &self.index
    }

    pub fn index_mut(&mut self) -> &mut SearchIndex {
        &mut self.index
    }
}

#[cfg(test)]
mod tests {
    use super::local::{fuzzy_match, score_match};
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        assert!(fuzzy_match("hello", "hello"));
    }

    #[test]
    fn test_fuzzy_match_subsequence() {
        assert!(fuzzy_match("hlo", "hello"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("HELLO", "hello"));
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert!(!fuzzy_match("xyz", "hello"));
    }

    #[test]
    fn test_fuzzy_match_empty_query() {
        assert!(fuzzy_match("", "anything"));
    }

    #[test]
    fn test_fuzzy_match_query_longer() {
        assert!(!fuzzy_match("hello world", "hello"));
    }

    #[test]
    fn test_score_match_exact() {
        assert!((score_match("hello", "hello") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_score_match_prefix() {
        assert!((score_match("hel", "hello") - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_score_match_contains() {
        assert!((score_match("ell", "hello") - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_score_match_fuzzy() {
        assert!((score_match("hlo", "hello") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_score_match_no_match() {
        assert!((score_match("xyz", "hello") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_search_index_add_and_get() {
        let mut index = SearchIndex::new();
        let item = SearchItem::new("1", "Test App", "A test app", SearchCategory::App);
        index.add(item);
        assert!(index.get("1").is_some());
        assert_eq!(index.count(), 1);
    }

    #[test]
    fn test_search_index_remove() {
        let mut index = SearchIndex::new();
        index.add(SearchItem::new("1", "Test", "desc", SearchCategory::App));
        let removed = index.remove("1");
        assert!(removed.is_some());
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_search_index_by_category() {
        let mut index = SearchIndex::new();
        index.add(SearchItem::new("1", "App1", "desc", SearchCategory::App));
        index.add(SearchItem::new("2", "File1", "desc", SearchCategory::File));
        index.add(SearchItem::new("3", "App2", "desc", SearchCategory::App));

        let apps = index.by_category(&SearchCategory::App);
        assert_eq!(apps.len(), 2);
        let files = index.by_category(&SearchCategory::File);
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_search_launcher_search_exact() {
        let mut launcher = SearchLauncher::new();
        launcher.register_app("calc", "Calculator", "Simple calculator app");
        launcher.register_file("doc", "Document.txt", "A text document", "/path/doc.txt");

        let results = launcher.search("Calculator");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.id, "calc");
    }

    #[test]
    fn test_search_launcher_search_fuzzy() {
        let mut launcher = SearchLauncher::new();
        launcher.register_app("calc", "Calculator", "Simple calculator app");

        let results = launcher.search("clcltr");
        assert!(!results.is_empty());
        assert_eq!(results[0].1.id, "calc");
    }

    #[test]
    fn test_search_launcher_search_empty() {
        let mut launcher = SearchLauncher::new();
        launcher.register_app("calc", "Calculator", "Simple calculator app");
        let results = launcher.search("");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_launcher_search_no_match() {
        let launcher = SearchLauncher::new();
        let results = launcher.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_launcher_search_by_category() {
        let mut launcher = SearchLauncher::new();
        launcher.register_app("calc", "Calculator", "Calc app");
        launcher.register_command("ls", "List files", "List directory contents");

        let results = launcher.search_by_category("calc", &SearchCategory::Command);
        assert!(results.is_empty());

        let results = launcher.search_by_category("calc", &SearchCategory::App);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_launcher_hotkey() {
        let launcher = SearchLauncher::with_hotkey("Ctrl+P");
        assert_eq!(launcher.hotkey(), "Ctrl+P");
    }

    #[test]
    fn test_search_launcher_default_hotkey() {
        let launcher = SearchLauncher::new();
        assert_eq!(launcher.hotkey(), "Alt+Space");
    }

    #[test]
    fn test_search_launcher_toggle() {
        let mut launcher = SearchLauncher::new();
        assert!(launcher.is_enabled());
        launcher.set_enabled(false);
        assert!(!launcher.is_enabled());
    }

    #[test]
    fn test_search_item_with_keywords() {
        let item = SearchItem::new("1", "Test", "desc", SearchCategory::App)
            .with_keywords(vec!["keyword1".into(), "keyword2".into()]);
        assert_eq!(item.keywords.len(), 2);
    }

    #[test]
    fn test_search_launcher_search_keywords() {
        let mut launcher = SearchLauncher::new();
        let item = SearchItem::new("1", "My App", "An application", SearchCategory::App)
            .with_keywords(vec!["utility".into(), "tool".into()])
            .with_score(0.8);
        launcher.index_mut().add(item);

        let results = launcher.search("utility");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_category_as_str() {
        assert_eq!(SearchCategory::App.as_str(), "app");
        assert_eq!(SearchCategory::File.as_str(), "file");
        assert_eq!(SearchCategory::Command.as_str(), "command");
        assert_eq!(SearchCategory::AgentSkill.as_str(), "agent_skill");
    }

    #[test]
    fn test_search_launcher_register_agent_skill() {
        let mut launcher = SearchLauncher::new();
        launcher.register_agent_skill("code-review", "Code Review", "Review code quality");
        let results = launcher.search_by_category("", &SearchCategory::AgentSkill);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.id, "code-review");
    }

    #[test]
    fn test_search_index_all_sorted_by_score() {
        let mut index = SearchIndex::new();
        index.add(SearchItem::new("a", "A", "desc", SearchCategory::App).with_score(0.5));
        index.add(SearchItem::new("b", "B", "desc", SearchCategory::App).with_score(1.0));
        let all = index.all();
        assert_eq!(all[0].id, "b");
        assert_eq!(all[1].id, "a");
    }

    #[test]
    fn test_search_results_scored() {
        let mut launcher = SearchLauncher::new();
        launcher.register_app("exact-match", "ExactMatch", "desc");
        launcher.register_app("prefix-item", "PrefixMatch", "desc");
        let results = launcher.search("ExactMatch");
        assert_eq!(results[0].1.id, "exact-match");
    }
}
