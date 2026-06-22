use std::collections::HashMap;

use crate::core::error::MornError;

use super::def::CapabilityDef;

pub struct CapabilityRegistry {
    categories: HashMap<String, Vec<CapabilityDef>>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        let mut reg = CapabilityRegistry {
            categories: HashMap::new(),
        };
        reg.register_builtins();
        reg
    }

    fn register_builtins(&mut self) {
        for cat in &[
            "channel",
            "tool",
            "knowledge",
            "protocol",
            "theme",
            "ui_panel",
        ] {
            self.categories.entry(cat.to_string()).or_default();
        }
    }

    pub fn register(&mut self, def: CapabilityDef) -> Result<(), MornError> {
        let cat = self.categories.entry(def.category.clone()).or_default();
        if cat.iter().any(|c| c.id == def.id) {
            return Err(MornError::Internal(format!(
                "capability '{}' already registered in category '{}'",
                def.id, def.category
            )));
        }
        cat.push(def);
        Ok(())
    }

    pub fn list(&self, category: &str) -> Vec<&CapabilityDef> {
        self.categories
            .get(category)
            .map_or(vec![], |v| v.iter().collect())
    }

    pub fn list_categories(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_by_id(&self, id: &str) -> Option<&CapabilityDef> {
        self.categories
            .values()
            .flatten()
            .find(|c| c.id == id)
    }

    pub fn unregister(&mut self, category: &str, id: &str) -> Option<CapabilityDef> {
        self.categories
            .get_mut(category)?
            .iter()
            .position(|c| c.id == id)
            .and_then(|idx| Some(self.categories.get_mut(category)?.remove(idx)))
    }

    pub fn all(&self) -> Vec<&CapabilityDef> {
        self.categories.values().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capability::CapabilityDef;

    fn sample_def(category: &str, id: &str) -> CapabilityDef {
        CapabilityDef {
            id: id.to_string(),
            category: category.to_string(),
            name: "Test".into(),
            description: "desc".into(),
            config_schema: serde_json::Value::Object(Default::default()),
            entry: "test".into(),
            version: "1.0.0".into(),
            author: "author".into(),
        }
    }

    #[test]
    fn new_has_builtin_categories() {
        let reg = CapabilityRegistry::new();
        let cats = reg.list_categories();
        assert!(cats.contains(&"channel"));
        assert!(cats.contains(&"tool"));
        assert!(cats.contains(&"knowledge"));
        assert!(cats.contains(&"protocol"));
        assert!(cats.contains(&"theme"));
        assert!(cats.contains(&"ui_panel"));
    }

    #[test]
    fn register_and_list() {
        let mut reg = CapabilityRegistry::new();
        let def = sample_def("channel", "morn:tg");
        reg.register(def).unwrap();
        let items = reg.list("channel");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "morn:tg");
    }

    #[test]
    fn register_duplicate_fails() {
        let mut reg = CapabilityRegistry::new();
        let def = sample_def("tool", "morn:search");
        reg.register(def).unwrap();
        let dup = sample_def("tool", "morn:search");
        let result = reg.register(dup);
        assert!(result.is_err());
    }

    #[test]
    fn register_new_category() {
        let mut reg = CapabilityRegistry::new();
        let def = sample_def("blockchain", "morn:eth");
        reg.register(def).unwrap();
        let items = reg.list("blockchain");
        assert_eq!(items.len(), 1);
        assert!(reg.list_categories().contains(&"blockchain"));
    }

    #[test]
    fn get_by_id_across_categories() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_def("channel", "morn:tg")).unwrap();
        reg.register(sample_def("tool", "morn:search")).unwrap();
        assert!(reg.get_by_id("morn:tg").is_some());
        assert!(reg.get_by_id("morn:search").is_some());
        assert!(reg.get_by_id("nonexistent").is_none());
    }

    #[test]
    fn unregister_removes_capability() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_def("channel", "morn:tg")).unwrap();
        let removed = reg.unregister("channel", "morn:tg");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "morn:tg");
        assert!(reg.list("channel").is_empty());
    }

    #[test]
    fn unregister_nonexistent() {
        let mut reg = CapabilityRegistry::new();
        assert!(reg.unregister("channel", "nope").is_none());
        assert!(reg.unregister("nonexistent", "x").is_none());
    }

    #[test]
    fn all_returns_flattened() {
        let mut reg = CapabilityRegistry::new();
        reg.register(sample_def("channel", "morn:tg")).unwrap();
        reg.register(sample_def("tool", "morn:search")).unwrap();
        assert_eq!(reg.all().len(), 2);
    }

    #[test]
    fn list_nonexistent_category() {
        let reg = CapabilityRegistry::new();
        assert!(reg.list("nonexistent").is_empty());
    }
}