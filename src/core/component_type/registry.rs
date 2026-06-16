//! Component type registry.
use crate::core::error::MornError;
use std::collections::HashMap;

use super::def::ComponentTypeDef;

pub struct TypeRegistry {
    types: HashMap<String, ComponentTypeDef>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = TypeRegistry {
            types: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        let builtins = vec![
            ("tool", vec!["execute"], vec![]),
            ("knowledge", vec!["retrieve", "store"], vec![]),
            ("skill", vec!["execute"], vec!["tool"]),
            ("persona", vec!["generate", "embed"], vec![]),
            ("memory", vec!["store", "recall"], vec![]),
            ("model", vec!["predict", "embed"], vec![]),
            ("agent", vec!["chat", "act"], vec!["model", "skill"]),
            ("pipeline", vec!["run", "compose"], vec!["agent"]),
        ];

        for (name, interfaces, implements) in builtins {
            self.types.insert(
                name.to_string(),
                ComponentTypeDef {
                    type_name: name.to_string(),
                    interfaces: interfaces.into_iter().map(String::from).collect(),
                    config_schema: serde_json::json!({}),
                    implements: implements.into_iter().map(String::from).collect(),
                    author: "system".to_string(),
                    version: "1.0.0".to_string(),
                },
            );
        }
    }

    pub fn register(&mut self, def: ComponentTypeDef) -> Result<(), MornError> {
        if def.type_name.trim().is_empty() {
            return Err(MornError::Internal("type_name cannot be empty".to_string()))
        }
        if self.types.contains_key(&def.type_name) {
            return Err(MornError::Internal(format!("type '{}' is already registered", def.type_name)));
        }
        self.types.insert(def.type_name.clone(), def);
        Ok(())
    }

    pub fn get(&self, type_name: &str) -> Option<&ComponentTypeDef> {
        self.types.get(type_name)
    }

    pub fn list(&self) -> Vec<&ComponentTypeDef> {
        self.types.values().collect()
    }

    pub fn has(&self, type_name: &str) -> bool {
        self.types.contains_key(type_name)
    }

    pub fn available_interfaces(&self) -> Vec<String> {
        let mut ifaces: Vec<String> = self
            .types
            .values()
            .flat_map(|def| def.interfaces.clone())
            .collect();
        ifaces.sort();
        ifaces.dedup();
        ifaces
    }

    pub fn find_by_interface(&self, interface: &str) -> Vec<&ComponentTypeDef> {
        self.types
            .values()
            .filter(|def| def.interfaces.contains(&interface.to_string()))
            .collect()
    }

    pub fn unregister(&mut self, type_name: &str) -> bool {
        if type_name == "system" || self.is_builtin(type_name) {
            return false;
        }
        self.types.remove(type_name).is_some()
    }

    fn is_builtin(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "tool" | "knowledge" | "skill" | "persona" | "memory" | "model" | "agent" | "pipeline"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_registry_has_eight_builtin_types() {
        let registry = TypeRegistry::new();
        assert_eq!(registry.list().len(), 8);
    }

    #[test]
    fn list_contains_all_builtin_names() {
        let registry = TypeRegistry::new();
        let names: Vec<&str> = registry
            .list()
            .iter()
            .map(|d| d.type_name.as_str())
            .collect();
        for n in &[
            "tool",
            "knowledge",
            "skill",
            "persona",
            "memory",
            "model",
            "agent",
            "pipeline",
        ] {
            assert!(names.contains(n), "missing builtin type '{}'", n);
        }
    }

    #[test]
    fn register_adds_new_type() {
        let mut registry = TypeRegistry::new();
        let def = ComponentTypeDef {
            type_name: "vision_model".to_string(),
            interfaces: vec!["predict".to_string(), "embed".to_string()],
            config_schema: json!({"type": "object"}),
            implements: vec!["model".to_string()],
            author: "test".to_string(),
            version: "0.1.0".to_string(),
        };
        registry.register(def).unwrap();
        assert!(registry.has("vision_model"));
        assert_eq!(registry.list().len(), 9);
    }

    #[test]
    fn register_rejects_empty_name() {
        let mut registry = TypeRegistry::new();
        let def = ComponentTypeDef {
            type_name: "".to_string(),
            interfaces: vec![],
            config_schema: json!({}),
            implements: vec![],
            author: "test".to_string(),
            version: "0.1.0".to_string(),
        };
        assert!(registry.register(def).is_err());
    }

    #[test]
    fn register_rejects_duplicate() {
        let mut registry = TypeRegistry::new();
        let def = ComponentTypeDef {
            type_name: "tool".to_string(),
            interfaces: vec![],
            config_schema: json!({}),
            implements: vec![],
            author: "test".to_string(),
            version: "0.1.0".to_string(),
        };
        assert!(registry.register(def).is_err());
    }

    #[test]
    fn get_returns_none_for_unknown() {
        let registry = TypeRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn get_returns_some_for_builtin() {
        let registry = TypeRegistry::new();
        let def = registry.get("agent").unwrap();
        assert_eq!(def.type_name, "agent");
    }

    #[test]
    fn has_returns_false_for_unknown() {
        let registry = TypeRegistry::new();
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn available_interfaces_contains_all() {
        let registry = TypeRegistry::new();
        let ifaces = registry.available_interfaces();
        for expected in &[
            "execute", "retrieve", "store", "generate", "embed", "recall", "predict", "chat",
            "act", "run", "compose",
        ] {
            assert!(
                ifaces.contains(&expected.to_string()),
                "missing interface '{}'",
                expected
            );
        }
    }

    #[test]
    fn find_by_interface_returns_matching_types() {
        let registry = TypeRegistry::new();
        let results = registry.find_by_interface("predict");
        let names: Vec<&str> = results.iter().map(|d| d.type_name.as_str()).collect();
        assert!(names.contains(&"model"));
    }
}
