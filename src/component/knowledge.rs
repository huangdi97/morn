use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub key: String,
    pub value: String,
    pub source: String,
}

pub trait Knowledge: IOComponent {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String>;
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String>;
}

#[allow(dead_code)]
pub struct StaticKnowledge {
    id: String,
    name: String,
    data: HashMap<String, String>,
}

impl StaticKnowledge {
    pub fn new(data: HashMap<String, String>) -> Self {
        StaticKnowledge {
            id: "knowledge-static".into(),
            name: "Static Knowledge".into(),
            data,
        }
    }
}

impl Component for StaticKnowledge {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "knowledge"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for StaticKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "query key".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "knowledge value".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for StaticKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Knowledge for StaticKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let mut results = Vec::new();
        for (k, v) in &self.data {
            if k.contains(query) || query.is_empty() {
                results.push(KnowledgeItem {
                    key: k.clone(),
                    value: v.clone(),
                    source: "static".into(),
                });
            }
        }
        Ok(results)
    }
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String> {
        for item in items {
            self.data.insert(item.key, item.value);
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct FileKnowledge {
    id: String,
    name: String,
    file_path: String,
    data: HashMap<String, String>,
}

impl FileKnowledge {
    pub fn new(file_path: &str) -> Self {
        FileKnowledge {
            id: "knowledge-file".into(),
            name: "File Knowledge".into(),
            file_path: file_path.to_string(),
            data: HashMap::new(),
        }
    }
}

impl Component for FileKnowledge {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "knowledge"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for FileKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "query key".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "knowledge value".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for FileKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Knowledge for FileKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let mut results = Vec::new();
        for (k, v) in &self.data {
            if k.contains(query) || query.is_empty() {
                results.push(KnowledgeItem {
                    key: k.clone(),
                    value: v.clone(),
                    source: self.file_path.clone(),
                });
            }
        }
        Ok(results)
    }
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), String> {
        for item in items {
            self.data.insert(item.key, item.value);
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct SqliteKnowledge {
    id: String,
    name: String,
    conn: Option<crate::core::storage::Storage>,
    table_name: String,
}

impl SqliteKnowledge {
    pub fn new(storage: Option<crate::core::storage::Storage>, table_name: &str) -> Self {
        SqliteKnowledge {
            id: "knowledge-sqlite".into(),
            name: "SQLite Knowledge".into(),
            conn: storage,
            table_name: table_name.to_string(),
        }
    }
}

impl Component for SqliteKnowledge {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "knowledge"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for SqliteKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "query".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "query key".into(),
            },
            Port {
                id: "result".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "knowledge value".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for SqliteKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Knowledge for SqliteKnowledge {
    fn query(&self, _query: &str) -> Result<Vec<KnowledgeItem>, String> {
        Ok(vec![])
    }
    fn update(&mut self, _items: Vec<KnowledgeItem>) -> Result<(), String> {
        Ok(())
    }
}

pub fn create_default_knowledge() -> Vec<Box<dyn Knowledge>> {
    let mut static_data = HashMap::new();
    static_data.insert("stock_code_AAPL".into(), "AAPL".into());
    static_data.insert(
        "api_endpoint_deepseek".into(),
        "https://api.deepseek.com".into(),
    );

    vec![Box::new(StaticKnowledge::new(static_data))]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_static() -> StaticKnowledge {
        let mut data = HashMap::new();
        data.insert("key1".into(), "value1".into());
        data.insert("key2".into(), "value2".into());
        StaticKnowledge::new(data)
    }

    #[test]
    fn test_static_knowledge_new() {
        let k = make_static();
        assert_eq!(k.id(), "knowledge-static");
        assert_eq!(k.type_name(), "knowledge");
    }

    #[test]
    fn test_static_knowledge_query_match() {
        let k = make_static();
        let results = k.query("key1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "key1");
        assert_eq!(results[0].value, "value1");
        assert_eq!(results[0].source, "static");
    }

    #[test]
    fn test_static_knowledge_query_no_match() {
        let k = make_static();
        let results = k.query("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_static_knowledge_query_empty_returns_all() {
        let k = make_static();
        let results = k.query("").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_static_knowledge_update_insert() {
        let mut k = make_static();
        let items = vec![KnowledgeItem {
            key: "key3".into(),
            value: "value3".into(),
            source: "test".into(),
        }];
        k.update(items).unwrap();
        let results = k.query("key3").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "value3");
    }

    #[test]
    fn test_static_knowledge_update_overwrite() {
        let mut k = make_static();
        let items = vec![KnowledgeItem {
            key: "key1".into(),
            value: "overwritten".into(),
            source: "test".into(),
        }];
        k.update(items).unwrap();
        let results = k.query("key1").unwrap();
        assert_eq!(results[0].value, "overwritten");
    }

    #[test]
    fn test_static_knowledge_component_lifecycle() {
        let mut k = make_static();
        assert!(k.init().is_ok());
        assert!(k.run().is_ok());
        assert!(k.pause().is_ok());
        assert!(k.stop().is_ok());
        assert_eq!(k.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn test_file_knowledge_new() {
        let k = FileKnowledge::new("/tmp/test.json");
        assert_eq!(k.id(), "knowledge-file");
        assert_eq!(k.type_name(), "knowledge");
    }

    #[test]
    fn test_file_knowledge_query_empty() {
        let k = FileKnowledge::new("/tmp/test.json");
        let results = k.query("anything").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_file_knowledge_update_and_query() {
        let mut k = FileKnowledge::new("/tmp/test.json");
        let items = vec![
            KnowledgeItem {
                key: "a".into(),
                value: "1".into(),
                source: "test".into(),
            },
            KnowledgeItem {
                key: "b".into(),
                value: "2".into(),
                source: "test".into(),
            },
        ];
        k.update(items).unwrap();
        let results = k.query("a").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "1");
        assert_eq!(results[0].source, "/tmp/test.json");
    }

    #[test]
    fn test_file_knowledge_query_empty_returns_all() {
        let mut k = FileKnowledge::new("/tmp/test.json");
        let items = vec![KnowledgeItem {
            key: "x".into(),
            value: "y".into(),
            source: "test".into(),
        }];
        k.update(items).unwrap();
        let results = k.query("").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_file_knowledge_requires_read_permission() {
        let k = FileKnowledge::new("/tmp/test.json");
        let perms = k.required_permissions();
        assert_eq!(perms.len(), 1);
        assert!(matches!(perms[0], Permission::ReadFile));
    }

    #[test]
    fn test_sqlite_knowledge_new() {
        let k = SqliteKnowledge::new(None, "test_table");
        assert_eq!(k.id(), "knowledge-sqlite");
        assert_eq!(k.type_name(), "knowledge");
    }

    #[test]
    fn test_sqlite_knowledge_query_returns_empty() {
        let k = SqliteKnowledge::new(None, "test_table");
        let results = k.query("anything").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_sqlite_knowledge_update_noop() {
        let mut k = SqliteKnowledge::new(None, "test_table");
        let items = vec![KnowledgeItem {
            key: "k".into(),
            value: "v".into(),
            source: "test".into(),
        }];
        assert!(k.update(items).is_ok());
    }

    #[test]
    fn test_create_default_knowledge() {
        let items = create_default_knowledge();
        assert_eq!(items.len(), 1);
        let results = items[0].query("AAPL").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "AAPL");
    }
}
