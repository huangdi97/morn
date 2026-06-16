//! support — Provides workflow templates for support and troubleshooting tasks.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

impl WorkflowTemplate {
    pub(super) fn decision_eval_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-decision-eval".into(),
            name: "Decision Evaluation".into(),
            description: "Multi-perspective decision evaluation for project proposals".into(),
            category: "strategy".into(),
            tags: vec!["decision".into(), "evaluation".into(), "strategy".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 75,
            steps: vec![
                WorkflowStep {
                    id: "proposal_analysis".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze the proposal and extract key parameters.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec![],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "risk_assessment".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "analyst".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["proposal_analysis".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "resource_eval".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "calc".into(),
                        params: serde_json::json!({"expression": ""}),
                    },
                    depends_on: vec!["proposal_analysis".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "stakeholder_impact".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "researcher".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["risk_assessment".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "recommendation".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Synthesize all evaluations into a go/no-go recommendation."
                            .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["resource_eval".into(), "stakeholder_impact".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "present".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "Decision evaluation complete".into(),
                    },
                    depends_on: vec!["recommendation".into()],
                    timeout_secs: 5,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_eval_has_six_steps() {
        let t = WorkflowTemplate::decision_eval_template();
        assert_eq!(t.id, "workflow-decision-eval");
        assert_eq!(t.name, "Decision Evaluation");
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn decision_eval_step_ids_in_order() {
        let t = WorkflowTemplate::decision_eval_template();
        let ids: Vec<&str> = t.steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "proposal_analysis",
                "risk_assessment",
                "resource_eval",
                "stakeholder_impact",
                "recommendation",
                "present"
            ]
        );
    }

    #[test]
    fn decision_eval_step_actions_are_diverse() {
        let t = WorkflowTemplate::decision_eval_template();
        assert!(matches!(t.steps[0].action, WorkflowAction::LLMCall { .. }));
        assert!(matches!(
            t.steps[1].action,
            WorkflowAction::AgentCall { .. }
        ));
        assert!(matches!(t.steps[2].action, WorkflowAction::ToolCall { .. }));
        assert!(matches!(
            t.steps[5].action,
            WorkflowAction::Notification { .. }
        ));
    }

    #[test]
    fn decision_eval_dependency_chain_is_correct() {
        let t = WorkflowTemplate::decision_eval_template();
        assert!(t.steps[1].depends_on.contains(&"proposal_analysis".into()));
        assert!(t.steps[2].depends_on.contains(&"proposal_analysis".into()));
        assert!(t.steps[3].depends_on.contains(&"risk_assessment".into()));
        assert!(t.steps[4].depends_on.contains(&"resource_eval".into()));
        assert!(t.steps[4].depends_on.contains(&"stakeholder_impact".into()));
    }

    #[test]
    fn decision_eval_ends_with_notification() {
        let t = WorkflowTemplate::decision_eval_template();
        assert!(matches!(
            t.steps[5].action,
            WorkflowAction::Notification { .. }
        ));
    }

    #[test]
    fn decision_eval_no_approval_required() {
        let t = WorkflowTemplate::decision_eval_template();
        for step in &t.steps {
            assert!(!step.approval_required);
        }
    }

    #[test]
    fn decision_eval_category_and_tags_are_set() {
        let t = WorkflowTemplate::decision_eval_template();
        assert_eq!(t.category, "strategy");
        assert!(t.tags.contains(&"decision".into()));
        assert!(t.tags.contains(&"evaluation".into()));
    }
}
