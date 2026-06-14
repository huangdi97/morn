//! templates — Collects built-in workflow templates by task category.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};

mod business;
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
            Self::crm_template(),
            Self::invoice_template(),
            Self::email_marketing_template(),
            Self::client_portal_template(),
            Self::schedule_template(),
        ]
    }

    pub fn get_by_id(id: &str) -> Option<WorkflowTemplate> {
        Self::list_builtin().into_iter().find(|t| t.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_builtin_returns_thirteen_templates() {
        let templates = WorkflowTemplate::list_builtin();
        assert_eq!(templates.len(), 13);
    }

    #[test]
    fn get_by_id_finds_existing_template() {
        let t = WorkflowTemplate::get_by_id("workflow-task-execution");
        assert!(t.is_some());
        assert_eq!(t.unwrap().id, "workflow-task-execution");
    }

    #[test]
    fn get_by_id_returns_none_for_missing_id() {
        let t = WorkflowTemplate::get_by_id("non-existent-id");
        assert!(t.is_none());
    }

    #[test]
    fn every_template_has_non_empty_id_and_category() {
        for t in WorkflowTemplate::list_builtin() {
            assert!(!t.id.is_empty(), "template id is empty");
            assert!(
                !t.category.is_empty(),
                "template category is empty for id={}",
                t.id
            );
        }
    }
}
