//! local — Searches local apps, files, and commands for the launcher.
use super::{SearchCategory, SearchItem, SearchLauncher};

pub(super) fn fuzzy_match(query: &str, target: &str) -> bool {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    if query.is_empty() {
        return true;
    }
    if query.len() > target.len() {
        return false;
    }

    let mut qi = query.chars().peekable();
    for tc in target.chars() {
        if let Some(&qc) = qi.peek() {
            if tc == qc {
                qi.next();
            }
        }
    }
    qi.next().is_none()
}

pub(super) fn score_match(query: &str, target: &str) -> f64 {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    if target == query {
        return 1.0;
    }
    if target.starts_with(&query) {
        return 0.9;
    }
    if target.contains(&query) {
        return 0.7;
    }
    if fuzzy_match(&query, &target) {
        return 0.5;
    }
    0.0
}

impl SearchLauncher {
    pub fn search(&self, query: &str) -> Vec<(f64, &SearchItem)> {
        if query.is_empty() {
            return self
                .index
                .all()
                .into_iter()
                .map(|item| (item.score, item))
                .collect();
        }

        let mut results: Vec<(f64, &SearchItem)> = self
            .index
            .all()
            .into_iter()
            .filter_map(|item| {
                let mut best = 0.0_f64;

                let s = score_match(query, &item.name);
                if s > best {
                    best = s;
                }

                let s = score_match(query, &item.description);
                if s > best {
                    best = s;
                }

                for kw in &item.keywords {
                    let s = score_match(query, kw);
                    if s > best {
                        best = s;
                    }
                }

                if best > 0.0 {
                    Some((best * item.score.max(0.1), item))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn search_by_category(
        &self,
        query: &str,
        category: &SearchCategory,
    ) -> Vec<(f64, &SearchItem)> {
        let items = self.index.by_category(category);
        if query.is_empty() {
            return items.into_iter().map(|item| (item.score, item)).collect();
        }

        let mut results: Vec<(f64, &SearchItem)> = items
            .into_iter()
            .filter_map(|item| {
                let mut best = 0.0_f64;

                let s = score_match(query, &item.name);
                if s > best {
                    best = s;
                }

                let s = score_match(query, &item.description);
                if s > best {
                    best = s;
                }

                for kw in &item.keywords {
                    let s = score_match(query, kw);
                    if s > best {
                        best = s;
                    }
                }

                if best > 0.0 {
                    Some((best * item.score.max(0.1), item))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn register_app(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        let item = SearchItem::new(id, name, description, SearchCategory::App);
        self.index.add(item);
    }

    pub fn register_file(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        path: impl Into<String>,
    ) {
        let item = SearchItem::new(id, name, description, SearchCategory::File)
            .with_metadata("path", path);
        self.index.add(item);
    }

    pub fn register_command(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        let item = SearchItem::new(id, name, description, SearchCategory::Command);
        self.index.add(item);
    }
}
