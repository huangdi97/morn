//! repo_map — Builds repository maps for code understanding and navigation.
use crate::core::error::MornError;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepoNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<RepoNode>,
    pub size: u64,
    pub language: Option<String>,
}

impl RepoNode {
    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
}

pub struct RepoMap {
    root: String,
    tree: Vec<RepoNode>,
    max_depth: usize,
}

impl RepoMap {
    pub fn new(root: &str) -> Self {
        RepoMap {
            root: root.to_string(),
            tree: Vec::new(),
            max_depth: 5,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn scan(&mut self) -> Result<(), MornError> {
        let root_path = Path::new(&self.root);
        if !root_path.exists() {
            return Err(MornError::Internal(format!("Path does not exist: {}", self.root)));
        }
        self.tree = self.scan_dir(root_path, 0)?;
        Ok(())
    }

    fn scan_dir(&self, dir: &Path, depth: usize) -> Result<Vec<RepoNode>, MornError> {
        if depth > self.max_depth {
            return Ok(Vec::new());
        }

        let mut nodes = Vec::new();
        let entries = std::fs::read_dir(dir).map_err(|e| MornError::Internal(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| MornError::Internal(e.to_string()))?;
            let path = entry.path();

            let name = entry.file_name().to_str().unwrap_or("").to_string();

            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            let metadata = std::fs::metadata(&path).map_err(|e| MornError::Internal(e.to_string()))?;
            let size = metadata.len();

            if path.is_dir() {
                let children = self.scan_dir(&path, depth + 1)?;
                nodes.push(RepoNode {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: true,
                    children,
                    size: 0,
                    language: None,
                });
            } else {
                let language = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_string());
                nodes.push(RepoNode {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: false,
                    children: Vec::new(),
                    size,
                    language,
                });
            }
        }

        nodes.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        Ok(nodes)
    }

    pub fn summarize(&self, max_tokens: usize) -> String {
        let mut summary = String::new();
        self.write_tree(&self.tree, 0, &mut summary);
        if summary.len() > max_tokens * 4 {
            summary.truncate(max_tokens * 4);
            summary.push_str("\n... (truncated)");
        }
        summary
    }

    fn write_tree(&self, nodes: &[RepoNode], depth: usize, output: &mut String) {
        for node in nodes {
            let indent = "  ".repeat(depth);
            if node.is_dir {
                output.push_str(&format!("{}📁 {}/\n", indent, node.name));
                self.write_tree(&node.children, depth + 1, output);
            } else {
                let lang = node
                    .language
                    .as_ref()
                    .map(|l| format!(" [{}]", l))
                    .unwrap_or_default();
                output.push_str(&format!("{}📄 {}{}\n", indent, node.name, lang));
            }
        }
    }

    pub fn find_related(&self, query: &str) -> Vec<&RepoNode> {
        let mut results = Vec::new();
        let lower_query = query.to_lowercase();
        self.search_nodes(&self.tree, &lower_query, &mut results);
        results
    }

    fn search_nodes<'a>(
        &'a self,
        nodes: &'a [RepoNode],
        query: &str,
        results: &mut Vec<&'a RepoNode>,
    ) {
        for node in nodes {
            if node.name.to_lowercase().contains(query) || node.path.to_lowercase().contains(query)
            {
                results.push(node);
            }
            if node.is_dir {
                self.search_nodes(&node.children, query, results);
            }
        }
    }

    pub fn file_count(&self) -> usize {
        self.count_files(&self.tree)
    }

    fn count_files(&self, nodes: &[RepoNode]) -> usize {
        let mut count = 0;
        for node in nodes {
            if node.is_file() {
                count += 1;
            }
            count += self.count_files(&node.children);
        }
        count
    }

    pub fn languages(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        self.collect_languages(&self.tree, &mut counts);
        counts
    }

    fn collect_languages(&self, nodes: &[RepoNode], counts: &mut HashMap<String, usize>) {
        for node in nodes {
            if let Some(ref lang) = node.language {
                *counts.entry(lang.clone()).or_insert(0) += 1;
            }
            if node.is_dir {
                self.collect_languages(&node.children, counts);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_repo_map() {
        let map = RepoMap::new("/tmp");
        assert_eq!(map.root, "/tmp");
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let mut map = RepoMap::new("/nonexistent/path");
        assert!(map.scan().is_err());
    }

    #[test]
    fn test_scan_current_dir() {
        let map = RepoMap::new(".").with_max_depth(2);
        let mut map = map;
        map.scan().unwrap();
        assert!(map.file_count() > 0);
    }

    #[test]
    fn test_summarize() {
        let map = RepoMap::new(".").with_max_depth(2);
        let mut map = map;
        map.scan().unwrap();
        let summary = map.summarize(2000);
        assert!(!summary.is_empty());
        assert!(summary.len() < 20000);
    }

    #[test]
    fn test_find_related() {
        let map = RepoMap::new(".").with_max_depth(3);
        let mut map = map;
        map.scan().unwrap();
        let results = map.find_related("Cargo");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_languages() {
        let map = RepoMap::new(".").with_max_depth(3);
        let mut map = map;
        map.scan().unwrap();
        let langs = map.languages();
        assert!(langs.contains_key("rs") || langs.contains_key("toml"));
    }

    #[test]
    fn test_empty_query() {
        let map = RepoMap::new(".").with_max_depth(1);
        let mut map = map;
        map.scan().unwrap();
        let results = map.find_related("");
        assert_ne!(results.len(), 0);
    }
}
