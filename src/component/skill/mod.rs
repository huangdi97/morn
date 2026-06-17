//! skill — Defines executable skills and their component integration.
use crate::core::component::{Data, IOComponent};
use crate::core::error::MornError;
use std::collections::HashMap;

mod builtins;
mod registry;
pub mod self_evolution;

pub use builtins::{CodeReviewSkill, DataAnalysisSkill, ReportGenSkill, WebResearchSkill};
pub use registry::create_default_skills;
pub use self_evolution::SelfEvolutionSkill;

#[derive(Debug, Clone)]
pub struct SkillStep {
    pub step_id: String,
    pub tool_id: String,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
    pub depends_on: Vec<String>,
    pub llm_step: bool,
    pub llm_prompt: Option<String>,
}

pub trait Skill: IOComponent {
    fn steps(&self) -> Vec<SkillStep>;
    fn execute(&mut self, input: Data) -> Result<Data, MornError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_load() {
        let skills = create_default_skills();
        assert_eq!(skills.len(), 5);

        let research = WebResearchSkill::new();
        assert_eq!(research.id, "skill-web-research");
        assert_eq!(research.name, "Web Research");
        assert_eq!(research.steps().len(), 2);

        let analysis = DataAnalysisSkill::new();
        assert_eq!(analysis.steps().len(), 3);

        let report = ReportGenSkill::new();
        assert!(report.steps().is_empty());

        let review = CodeReviewSkill::new();
        assert!(review.steps().is_empty());
    }

    #[test]
    fn test_skill_execute() {
        let mut skill = WebResearchSkill::new();
        let result = skill.execute(Data::text("machine learning")).unwrap();
        assert!(result.content.as_str().unwrap().contains("web_research"));
        assert!(result
            .content
            .as_str()
            .unwrap()
            .contains("machine learning"));
    }

    #[test]
    fn test_skill_invalid() {
        let mut skill = WebResearchSkill::new();
        let result = skill.execute(Data::text(""));
        assert!(result.is_ok());
    }
}
