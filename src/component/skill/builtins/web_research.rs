//! web_research — Defines the built-in skill for web research workflows.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use crate::core::error::MornError;

#[allow(dead_code)] /* 预留：内置 Web Research skill 注册入口 */
pub struct WebResearchSkill {
    pub(in crate::component::skill) id: String,
    pub(in crate::component::skill) name: String,
    steps: Vec<SkillStep>,
}

impl WebResearchSkill {
    pub fn new() -> Self {
        let steps = vec![
            SkillStep {
                step_id: "search".into(),
                tool_id: "web_search".into(),
                input_mapping: [("query".into(), "input.query".into())].into(),
                output_mapping: [("results".into(), "search_results".into())].into(),
                depends_on: vec![],
                llm_step: false,
                llm_prompt: None,
            },
            SkillStep {
                step_id: "summarize".into(),
                tool_id: "".into(),
                input_mapping: [("text".into(), "search_results".into())].into(),
                output_mapping: [("summary".into(), "output".into())].into(),
                depends_on: vec!["search".into()],
                llm_step: true,
                llm_prompt: Some("Summarize the following search results concisely:".into()),
            },
        ];
        WebResearchSkill {
            id: "skill-web-research".into(),
            name: "Web Research".into(),
            steps,
        }
    }
}

impl Default for WebResearchSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for WebResearchSkill {
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

impl IOComponent for WebResearchSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "research topic".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "research summary".into(),
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

impl SecureComponent for WebResearchSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Skill for WebResearchSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }
    fn execute(&mut self, input: Data) -> Result<Data, MornError> {
        let topic = input.content.as_str().unwrap_or("").to_string();
        let mut search_tool = crate::component::tool::get_tool_by_name("web_search")
            .ok_or("web_search tool not found")?;
        let search_result = search_tool.execute(Data::text(&topic))?;
        let summary = format!(
            "[web_research] summary for '{}': {}",
            topic,
            search_result.content.as_str().unwrap_or("")
        );
        Ok(Data::text(&summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_research_skill_has_expected_metadata_and_permissions() {
        let skill = WebResearchSkill::new();
        assert_eq!(skill.id(), "skill-web-research");
        assert_eq!(skill.type_name(), "skill");
        assert_eq!(skill.health_check(), HealthStatus::Healthy);
        assert_eq!(
            skill.required_permissions(),
            vec![Permission::NetworkAccess]
        );
    }

    #[test]
    fn web_research_steps_preserve_search_then_summarize_flow() {
        let skill = WebResearchSkill::new();
        let steps = skill.steps();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_id, "search");
        assert_eq!(steps[0].tool_id, "web_search");
        assert!(steps[0].depends_on.is_empty());
        assert_eq!(steps[1].depends_on, vec!["search"]);
        assert!(steps[1].llm_step);
    }

    #[test]
    fn web_research_execute_includes_topic_and_search_result() {
        let mut skill = WebResearchSkill::new();
        let result = skill.execute(Data::text("local first ai")).unwrap();
        let text = result.content.as_str().unwrap();
        assert!(text.contains("[web_research]"));
        assert!(text.contains("local first ai"));
        assert!(text.contains("[web_search]"));
    }
}
