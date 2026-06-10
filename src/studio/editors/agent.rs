#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentEditor {
    pub id: String,
    pub name: String,
}

impl AgentEditor {
    pub fn new(id: &str, name: &str) -> Self {
        AgentEditor {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    pub fn load() -> Self {
        AgentEditor {
            id: "default".into(),
            name: "Default Agent".into(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}
