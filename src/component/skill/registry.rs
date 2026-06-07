//! registry — Registers built-in and custom skills for lookup and execution.
use super::{CodeReviewSkill, DataAnalysisSkill, ReportGenSkill, Skill, WebResearchSkill};

pub fn create_default_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(WebResearchSkill::new()),
        Box::new(DataAnalysisSkill::new()),
        Box::new(ReportGenSkill::new()),
        Box::new(CodeReviewSkill::new()),
    ]
}
