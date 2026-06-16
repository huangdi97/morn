//! cache — Stores cached launcher search items for fast retrieval.
use crate::core::error::MornError;
use super::{SearchCategory, SearchItem};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchIndex {
    items: HashMap<String, SearchItem>,
    categories: HashMap<SearchCategory, Vec<String>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        SearchIndex {
            items: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn add(&mut self, item: SearchItem) {
        let id = item.id.clone();
        let category = item.category.clone();
        self.categories
            .entry(category)
            .or_default()
            .push(id.clone());
        self.items.insert(id, item);
    }

    pub fn remove(&mut self, id: &str) -> Option<SearchItem> {
        if let Some(item) = self.items.remove(id) {
            if let Some(ids) = self.categories.get_mut(&item.category) {
                ids.retain(|i| i != id);
            }
            Some(item)
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<&SearchItem> {
        self.items.get(id)
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn all(&self) -> Vec<&SearchItem> {
        let mut items: Vec<_> = self.items.values().collect();
        items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items
    }

    pub fn by_category(&self, category: &SearchCategory) -> Vec<&SearchItem> {
        self.categories
            .get(category)
            .map(|ids| {
                let mut items: Vec<_> = ids.iter().filter_map(|id| self.items.get(id)).collect();
                items.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                items
            })
            .unwrap_or_default()
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}
