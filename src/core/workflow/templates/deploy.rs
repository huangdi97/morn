//! deploy — Provides workflow templates for deployment tasks.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

impl WorkflowTemplate {
    pub(super) fn product_launch_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-product-launch".into(),
            name: "Product Launch".into(),
            description: "Product launch planning and execution workflow".into(),
            category: "product".into(),
            tags: vec!["product".into(), "launch".into(), "go-to-market".into()],
            estimated_duration_secs: 90,
            steps: vec![
                WorkflowStep {
                    id: "market_research".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "web_search".into(),
                        params: serde_json::json!({"query": ""}),
                    },
                    depends_on: vec![],
                    timeout_secs: 20,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "positioning".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Define product positioning and key messaging.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["market_research".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "launch_plan".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt:
                            "Create a comprehensive launch plan with timeline and milestones."
                                .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["positioning".into()],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "materials".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "writer".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["launch_plan".into()],
                    timeout_secs: 30,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "coordinate".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "Launch coordination".into(),
                    },
                    depends_on: vec!["materials".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "launch".into(),
                    action: WorkflowAction::HumanApproval {
                        message: "Ready to launch? Confirm to proceed.".into(),
                    },
                    depends_on: vec!["coordinate".into()],
                    timeout_secs: 600,
                    retry_count: 0,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn task_execution_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-task-execution".into(),
            name: "Task Execution".into(),
            description: "General task execution workflow with planning, execution, and review"
                .into(),
            category: "general".into(),
            tags: vec!["task".into(), "execution".into(), "general".into()],
            estimated_duration_secs: 30,
            steps: vec![
                WorkflowStep {
                    id: "understand".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze the user request and extract requirements.".into(),
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
                    id: "plan".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Create a step-by-step plan to fulfill the requirements."
                            .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["understand".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "execute".into(),
                    action: WorkflowAction::AgentCall {
                        agent_id: "default".into(),
                        input: "".into(),
                    },
                    depends_on: vec!["plan".into()],
                    timeout_secs: 30,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "review".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Review the execution result and verify completeness."
                            .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["execute".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "summarize".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Provide a concise summary of what was done.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["review".into()],
                    timeout_secs: 10,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "notify".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "Task completed".into(),
                    },
                    depends_on: vec!["summarize".into()],
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
