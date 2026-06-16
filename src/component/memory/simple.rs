//! simple — Provides an in-memory implementation of the memory component.
use crate::core::error::MornError;
use super::Memory;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use std::collections::HashMap;

#[allow(dead_code)] /* 预留：SQLite 持久化 memory 组件 */
pub struct SqliteMemory {
    id: String,
    name: String,
    data: HashMap<String, HashMap<String, String>>,
}

impl SqliteMemory {
    pub fn new() -> Self {
        SqliteMemory {
            id: "memory-sqlite".into(),
            name: "SQLite Memory".into(),
            data: HashMap::new(),
        }
    }
}

impl Default for SqliteMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SqliteMemory {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "memory"
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

impl IOComponent for SqliteMemory {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "store".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "key:value:namespace".into(),
            },
            Port {
                id: "retrieve".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "stored value".into(),
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

impl SecureComponent for SqliteMemory {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Memory for SqliteMemory {
    fn store(&mut self, key: &str, value: &str, namespace: &str) -> Result<(), MornError> {
        self.data
            .entry(namespace.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn retrieve(&self, key: &str, namespace: &str) -> Result<Option<String>, MornError> {
        Ok(self.data.get(namespace).and_then(|ns| ns.get(key)).cloned())
    }

    fn search(&self, query: &str, namespace: &str) -> Result<Vec<(String, String)>, MornError> {
        let mut results = Vec::new();
        if let Some(ns) = self.data.get(namespace) {
            for (k, v) in ns {
                if k.contains(query) || v.contains(query) {
                    results.push((k.clone(), v.clone()));
                }
            }
        }
        Ok(results)
    }
}

pub fn create_default_memory() -> Box<dyn Memory> {
    Box::new(SqliteMemory::new())
}
