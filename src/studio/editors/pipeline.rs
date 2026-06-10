#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineEditor {
    pub id: String,
    pub name: String,
}

impl PipelineEditor {
    pub fn new(id: &str, name: &str) -> Self {
        PipelineEditor {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    pub fn load() -> Self {
        PipelineEditor {
            id: "default".into(),
            name: "Default Pipeline".into(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}
