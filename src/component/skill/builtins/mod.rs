//! builtins — Collects built-in skills available to the skill registry.
mod code_review;
mod data_analysis;
mod report_generation;
mod web_research;

pub use code_review::CodeReviewSkill;
pub use data_analysis::DataAnalysisSkill;
pub use report_generation::ReportGenSkill;
pub use web_research::WebResearchSkill;
