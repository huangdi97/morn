//! data_analysis — Defines the built-in skill for data analysis workflows.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use crate::core::error::MornError;

#[allow(dead_code)] /* 预留：内置 Data Analysis skill 注册入口 */
pub struct DataAnalysisSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
}

impl DataAnalysisSkill {
    pub fn new() -> Self {
        let steps = vec![
            SkillStep {
                step_id: "read".into(),
                tool_id: "read_file".into(),
                input_mapping: [("path".into(), "input.path".into())].into(),
                output_mapping: [("data".into(), "raw_data".into())].into(),
                depends_on: vec![],
                llm_step: false,
                llm_prompt: None,
            },
            SkillStep {
                step_id: "analyze".into(),
                tool_id: "calc".into(),
                input_mapping: [("expression".into(), "raw_data".into())].into(),
                output_mapping: [("result".into(), "analysis".into())].into(),
                depends_on: vec!["read".into()],
                llm_step: true,
                llm_prompt: Some("Analyze the following data and provide insights:".into()),
            },
            SkillStep {
                step_id: "format".into(),
                tool_id: "".into(),
                input_mapping: [("text".into(), "analysis".into())].into(),
                output_mapping: [("report".into(), "output".into())].into(),
                depends_on: vec!["analyze".into()],
                llm_step: true,
                llm_prompt: Some("Format the analysis into a clear report:".into()),
            },
        ];
        DataAnalysisSkill {
            id: "skill-data-analysis".into(),
            name: "Data Analysis".into(),
            steps,
        }
    }
}

impl Default for DataAnalysisSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DataAnalysisSkill {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "skill"
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

impl IOComponent for DataAnalysisSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "file path".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "analysis report".into(),
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

impl SecureComponent for DataAnalysisSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Skill for DataAnalysisSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }
    fn execute(&mut self, input: Data) -> Result<Data, MornError> {
        let path = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[data_analysis] analysis of '{}' complete",
            path
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_analysis_skill_has_expected_metadata_and_permissions() {
        let skill = DataAnalysisSkill::new();
        assert_eq!(skill.id(), "skill-data-analysis");
        assert_eq!(skill.type_name(), "skill");
        assert_eq!(skill.health_check(), HealthStatus::Healthy);
        assert_eq!(skill.required_permissions(), vec![Permission::ReadFile]);
    }

    #[test]
    fn data_analysis_steps_preserve_pipeline_order() {
        let skill = DataAnalysisSkill::new();
        let steps = skill.steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].step_id, "read");
        assert_eq!(steps[1].depends_on, vec!["read"]);
        assert_eq!(steps[2].depends_on, vec!["analyze"]);
        assert!(steps[1].llm_step);
        assert!(steps[2].llm_prompt.is_some());
    }

    #[test]
    fn data_analysis_execute_includes_requested_path() {
        let mut skill = DataAnalysisSkill::new();
        let result = skill.execute(Data::text("data.csv")).unwrap();
        let text = result.content.as_str().unwrap();
        assert!(text.contains("[data_analysis]"));
        assert!(text.contains("data.csv"));
    }
}
