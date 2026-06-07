//! templates — Collects built-in workflow templates by task category.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};

mod code_review;
mod data_analysis;
mod deploy;
mod support;

impl WorkflowTemplate {
    pub fn list_builtin() -> Vec<WorkflowTemplate> {
        vec![
            Self::task_execution_template(),
            Self::deep_analysis_template(),
            Self::news_monitor_template(),
            Self::report_generation_template(),
            Self::code_delivery_template(),
            Self::product_launch_template(),
            Self::decision_eval_template(),
            Self::scheduled_inspection_template(),
        ]
    }

    pub fn get_by_id(id: &str) -> Option<WorkflowTemplate> {
        Self::list_builtin().into_iter().find(|t| t.id == id)
    }
}
