//! Persistent knowledge implementations (static, file, SQLite).

use crate::core::error::MornError;
use std::collections::HashMap;

use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

use super::{Knowledge, KnowledgeItem};

#[allow(dead_code)] /* 预留：静态知识存储后端 */
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
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
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
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), MornError> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, MornError> {
        Ok(None)
    }
}

impl SecureComponent for StaticKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Knowledge for StaticKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, MornError> {
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
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), MornError> {
        for item in items {
            self.data.insert(item.key, item.value);
        }
        Ok(())
    }
}

#[allow(dead_code)] /* 预留：文件知识存储后端 */
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
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
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
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), MornError> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, MornError> {
        Ok(None)
    }
}

impl SecureComponent for FileKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Knowledge for FileKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, MornError> {
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
    fn update(&mut self, items: Vec<KnowledgeItem>) -> Result<(), MornError> {
        for item in items {
            self.data.insert(item.key, item.value);
        }
        Ok(())
    }
}

#[allow(dead_code)] /* 预留：SQLite 知识存储后端 */
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
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
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
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), MornError> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, MornError> {
        Ok(None)
    }
}

impl SecureComponent for SqliteKnowledge {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Knowledge for SqliteKnowledge {
    fn query(&self, _query: &str) -> Result<Vec<KnowledgeItem>, MornError> {
        Ok(vec![])
    }
    fn update(&mut self, _items: Vec<KnowledgeItem>) -> Result<(), MornError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_knowledge_new_empty() {
        let k = StaticKnowledge::new(HashMap::new());
        assert_eq!(k.query("anything").unwrap().len(), 0);
    }

    #[test]
    fn test_static_knowledge_query_finds_matching() {
        let mut data = HashMap::new();
        data.insert("rust".into(), "systems language".into());
        data.insert("python".into(), "scripting language".into());
        let k = StaticKnowledge::new(data);
        let results = k.query("rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "rust");
        assert_eq!(results[0].value, "systems language");
    }

    #[test]
    fn test_static_knowledge_empty_query_returns_all() {
        let mut data = HashMap::new();
        data.insert("a".into(), "1".into());
        data.insert("b".into(), "2".into());
        let k = StaticKnowledge::new(data);
        assert_eq!(k.query("").unwrap().len(), 2);
    }

    #[test]
    fn test_static_knowledge_update_adds_items() {
        let mut data = HashMap::new();
        data.insert("k1".into(), "v1".into());
        let mut k = StaticKnowledge::new(data);
        k.update(vec![KnowledgeItem {
            key: "k2".into(),
            value: "v2".into(),
            source: "test".into(),
        }])
        .unwrap();
        let results = k.query("").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_static_knowledge_component_impl() {
        let mut k = StaticKnowledge::new(HashMap::new());
        assert_eq!(k.id(), "knowledge-static");
        assert_eq!(k.type_name(), "knowledge");
        assert!(k.init().is_ok());
        assert!(k.run().is_ok());
        assert!(k.pause().is_ok());
        assert!(k.stop().is_ok());
        assert_eq!(k.health_check(), HealthStatus::Healthy);
    }

    #[test]
    fn test_static_knowledge_io_ports() {
        let k = StaticKnowledge::new(HashMap::new());
        let ports = k.ports();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].id, "query");
        assert_eq!(ports[1].id, "result");
    }

    #[test]
    fn test_static_knowledge_secure_permissions() {
        let k = StaticKnowledge::new(HashMap::new());
        assert!(k.required_permissions().is_empty());
    }

    #[test]
    fn test_file_knowledge_new() {
        let k = FileKnowledge::new("/tmp/test.json");
        assert!(k.query("").unwrap().is_empty());
    }

    #[test]
    fn test_file_knowledge_query_and_update() {
        let mut k = FileKnowledge::new("/tmp/test.json");
        k.update(vec![KnowledgeItem {
            key: "k".into(),
            value: "v".into(),
            source: "test".into(),
        }])
        .unwrap();
        let results = k.query("k").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "/tmp/test.json");
    }

    #[test]
    fn test_file_knowledge_component_impl() {
        let mut k = FileKnowledge::new("/tmp/test.json");
        assert_eq!(k.id(), "knowledge-file");
        assert!(k.init().is_ok());
        assert!(k.run().is_ok());
    }

    #[test]
    fn test_file_knowledge_permissions() {
        let k = FileKnowledge::new("/tmp/test.json");
        assert!(k.required_permissions().contains(&Permission::ReadFile));
    }

    #[test]
    fn test_sqlite_knowledge_new() {
        let k = SqliteKnowledge::new(None, "test_table");
        assert_eq!(k.id(), "knowledge-sqlite");
        assert!(k.query("").unwrap().is_empty());
    }

    #[test]
    fn test_sqlite_knowledge_update() {
        let mut k = SqliteKnowledge::new(None, "test_table");
        assert!(k.update(vec![]).is_ok());
    }

    #[test]
    fn test_sqlite_knowledge_component_impl() {
        let mut k = SqliteKnowledge::new(None, "test_table");
        assert!(k.init().is_ok());
        assert!(k.run().is_ok());
        assert_eq!(k.health_check(), HealthStatus::Healthy);
    }
}
