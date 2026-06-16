//! registry — Registers built-in and custom skills for lookup and execution.
use crate::core::error::MornError;
use super::{
    CodeReviewSkill, DataAnalysisSkill, ReportGenSkill, SelfEvolutionSkill, Skill, WebResearchSkill,
};

pub fn create_default_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(WebResearchSkill::new()),
        Box::new(DataAnalysisSkill::new()),
        Box::new(ReportGenSkill::new()),
        Box::new(CodeReviewSkill::new()),
        Box::new(SelfEvolutionSkill::new()),
    ]
}
