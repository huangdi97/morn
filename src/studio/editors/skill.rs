#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillEditor {
    pub id: String,
    pub name: String,
}

impl SkillEditor {
    pub fn new(id: &str, name: &str) -> Self {
        SkillEditor {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    pub fn load() -> Self {
        SkillEditor {
            id: "default".into(),
            name: "Default Skill".into(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}
