#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonaEditor {
    pub id: String,
    pub name: String,
}

impl PersonaEditor {
    pub fn new(id: &str, name: &str) -> Self {
        PersonaEditor {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    pub fn load() -> Self {
        PersonaEditor {
            id: "default".into(),
            name: "Default Persona".into(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}
