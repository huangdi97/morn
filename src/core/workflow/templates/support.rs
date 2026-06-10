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
