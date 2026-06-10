//! code_review — Provides workflow templates for code review tasks.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

impl WorkflowTemplate {
    pub(super) fn code_delivery_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-code-delivery".into(),
            name: "Code Delivery".into(),
            description: "End-to-end software development workflow".into(),
            category: "development".into(),
            tags: vec!["code".into(), "development".into(), "delivery".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 120,
            steps: vec![
                WorkflowStep {
                    id: "requirements".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze requirements and create specification.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec![],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "design".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Design architecture and component structure.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["requirements".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "implement".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "coder".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["design".into()],
                    timeout_secs: 60,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "test".into(),
                    action: WorkflowAction::CodeExec {
                        language: "python".into(),
                        script: "".into(),
                    },
                    depends_on: vec!["implement".into()],
                    timeout_secs: 30,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "review".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "reviewer".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["test".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "deploy".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "Code ready for deployment".into(),
                    },
                    depends_on: vec!["review".into()],
                    timeout_secs: 5,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "document".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Generate documentation for the delivered code.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["deploy".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn scheduled_inspection_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-scheduled-inspection".into(),
            name: "Scheduled Inspection".into(),
            description: "Regular system health and performance inspection".into(),
            category: "operations".into(),
            tags: vec!["ops".into(), "inspection".into(), "health".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 30,
            steps: vec![
                WorkflowStep {
                    id: "health_check".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "http_request".into(),
                        params: serde_json::json!({"url": ""}),
                    },
                    depends_on: vec![],
                    timeout_secs: 15,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "analyze_metrics".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze system metrics and identify anomalies.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["health_check".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "report".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt:
                            "Generate inspection report with status and recommendations.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["analyze_metrics".into()],
                    timeout_secs: 10,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "alert_if_needed".into(),
                    action: WorkflowAction::Condition {
                        expression: "status != healthy".into(),
                        true_branch: vec![WorkflowStep {
                            id: "alert".into(),
                            action: WorkflowAction::Notification {
                                channel: "default".into(),
                                message: "Inspection found issues".into(),
                            },
                            depends_on: vec![],
                            timeout_secs: 5,
                            retry_count: 2,
                            approval_required: false,
                            input_mapping: HashMap::new(),
                            output_mapping: HashMap::new(),
                        }],
                        false_branch: vec![],
                    },
                    depends_on: vec!["report".into()],
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
