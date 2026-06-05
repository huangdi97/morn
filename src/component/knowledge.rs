use crate::core::component::{Component, Data, HealthStatus, IOComponent, Port, PortDirection, SecureComponent, Permission};
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

pub struct StaticKnowledge {
    id: String,
    name: String,
    data: HashMap<String, String>,
}

impl StaticKnowledge {
    pub fn new(data: HashMap<String, String>) -> Self {
        StaticKnowledge { id: "knowledge-static".into(), name: "Static Knowledge".into(), data }
    }
}

impl Component for StaticKnowledge {
    fn id(&self) -> &str { &self.id }
    fn type_name(&self) -> &str { "knowledge" }
    fn init(&mut self) -> Result<(), String> { Ok(()) }
    fn run(&mut self) -> Result<(), String> { Ok(()) }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn stop(&mut self) -> Result<(), String> { Ok(()) }
    fn health_check(&self) -> HealthStatus { HealthStatus::Healthy }
}

impl IOComponent for StaticKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port { id: "query".into(), direction: PortDirection::Input, data_type: "text".into(), description: "query key".into() },
            Port { id: "result".into(), direction: PortDirection::Output, data_type: "text".into(), description: "knowledge value".into() },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> { Ok(()) }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> { Ok(None) }
}

impl SecureComponent for StaticKnowledge {
    fn required_permissions(&self) -> Vec<Permission> { vec![] }
}

impl Knowledge for StaticKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let mut results = Vec::new();
        for (k, v) in &self.data {
            if k.contains(query) || query.is_empty() {
                results.push(KnowledgeItem {
                    key: k.clone(), value: v.clone(), source: "static".into(),
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
    fn id(&self) -> &str { &self.id }
    fn type_name(&self) -> &str { "knowledge" }
    fn init(&mut self) -> Result<(), String> { Ok(()) }
    fn run(&mut self) -> Result<(), String> { Ok(()) }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn stop(&mut self) -> Result<(), String> { Ok(()) }
    fn health_check(&self) -> HealthStatus { HealthStatus::Healthy }
}

impl IOComponent for FileKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port { id: "query".into(), direction: PortDirection::Input, data_type: "text".into(), description: "query key".into() },
            Port { id: "result".into(), direction: PortDirection::Output, data_type: "text".into(), description: "knowledge value".into() },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> { Ok(()) }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> { Ok(None) }
}

impl SecureComponent for FileKnowledge {
    fn required_permissions(&self) -> Vec<Permission> { vec![Permission::ReadFile] }
}

impl Knowledge for FileKnowledge {
    fn query(&self, query: &str) -> Result<Vec<KnowledgeItem>, String> {
        let mut results = Vec::new();
        for (k, v) in &self.data {
            if k.contains(query) || query.is_empty() {
                results.push(KnowledgeItem {
                    key: k.clone(), value: v.clone(), source: self.file_path.clone(),
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
    fn id(&self) -> &str { &self.id }
    fn type_name(&self) -> &str { "knowledge" }
    fn init(&mut self) -> Result<(), String> { Ok(()) }
    fn run(&mut self) -> Result<(), String> { Ok(()) }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn stop(&mut self) -> Result<(), String> { Ok(()) }
    fn health_check(&self) -> HealthStatus { HealthStatus::Healthy }
}

impl IOComponent for SqliteKnowledge {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port { id: "query".into(), direction: PortDirection::Input, data_type: "text".into(), description: "query key".into() },
            Port { id: "result".into(), direction: PortDirection::Output, data_type: "text".into(), description: "knowledge value".into() },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> { Ok(()) }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> { Ok(None) }
}

impl SecureComponent for SqliteKnowledge {
    fn required_permissions(&self) -> Vec<Permission> { vec![Permission::ReadFile] }
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
    static_data.insert("api_endpoint_deepseek".into(), "https://api.deepseek.com".into());

    vec![
        Box::new(StaticKnowledge::new(static_data)),
    ]
}