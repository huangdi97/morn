#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeSource {
    pub name: String,
    pub source_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEditor {
    pub name: String,
    pub data_sources: Vec<KnowledgeSource>,
    pub process_method: String,
    pub update_strategy: String,
}

impl KnowledgeEditor {
    pub fn new(name: &str) -> Self {
        KnowledgeEditor {
            name: name.to_string(),
            data_sources: Vec::new(),
            process_method: "embedding".to_string(),
            update_strategy: "manual".to_string(),
        }
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "knowledge",
            "name": self.name,
            "data_sources": self.data_sources,
            "process_method": self.process_method,
            "update_strategy": self.update_strategy,
        })
    }
}
