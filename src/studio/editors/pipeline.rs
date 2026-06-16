//! pipeline — Pipeline editor with stage management (add, update, remove, reorder).

use crate::core::error::MornError;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineStage {
    pub id: String,
    pub name: String,
    pub stage_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineEditor {
    pub id: String,
    pub name: String,
    pub stages: Vec<PipelineStage>,
}

impl PipelineEditor {
    pub fn new(id: &str, name: &str) -> Self {
        PipelineEditor {
            id: id.to_string(),
            name: name.to_string(),
            stages: Vec::new(),
        }
    }

    pub fn load() -> Self {
        PipelineEditor {
            id: "default".into(),
            name: "Default Pipeline".into(),
            stages: Vec::new(),
        }
    }

    pub fn save(&self) -> Result<(), MornError> {
        Ok(())
    }

    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
    }

    pub fn update_stage(
        &mut self,
        stage_id: &str,
        config: serde_json::Value,
    ) -> Result<(), MornError> {
        let stage = self
            .stages
            .iter_mut()
            .find(|stage| stage.id == stage_id)
            .ok_or_else(|| format!("stage '{}' not found", stage_id))?;
        stage.config = config;
        Ok(())
    }

    pub fn remove_stage(&mut self, stage_id: &str) -> Result<PipelineStage, MornError> {
        let index = self
            .stages
            .iter()
            .position(|stage| stage.id == stage_id)
            .ok_or_else(|| format!("stage '{}' not found", stage_id))?;
        Ok(self.stages.remove(index))
    }

    pub fn move_stage(&mut self, stage_id: &str, new_index: usize) -> Result<(), MornError> {
        if new_index >= self.stages.len() {
            return Err(MornError::Internal(format!("stage index {} out of range", new_index)));
        }
        let index = self
            .stages
            .iter()
            .position(|stage| stage.id == stage_id)
            .ok_or_else(|| format!("stage '{}' not found", stage_id))?;
        let stage = self.stages.remove(index);
        self.stages.insert(new_index, stage);
        Ok(())
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "pipeline",
            "id": self.id,
            "name": self.name,
            "stages": self.stages,
        })
    }
}
