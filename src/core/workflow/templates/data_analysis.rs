//! data_analysis — Provides workflow templates for data analysis tasks.
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

impl WorkflowTemplate {
    pub(super) fn deep_analysis_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-deep-analysis".into(),
            name: "Deep Analysis".into(),
            description: "Multi-source research and deep analysis workflow".into(),
            category: "research".into(),
            tags: vec!["analysis".into(), "research".into(), "data".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "gather".into(),
                    action: WorkflowAction::ToolCall { tool_id: "web_search".into(), params: serde_json::json!({"query": ""}) },
                    depends_on: vec![],
                    timeout_secs: 20, retry_count: 2, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "cross_verify".into(),
                    action: WorkflowAction::ToolCall { tool_id: "web_search".into(), params: serde_json::json!({"query": ""}) },
                    depends_on: vec!["gather".into()],
                    timeout_secs: 20, retry_count: 2, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "analyze".into(),
                    action: WorkflowAction::LLMCall { system_prompt: "Perform deep analysis on the gathered data. Identify patterns, correlations, and insights.".into(), model: "default".into() },
                    depends_on: vec!["cross_verify".into()],
                    timeout_secs: 30, retry_count: 1, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "conclude".into(),
                    action: WorkflowAction::LLMCall { system_prompt: "Draw conclusions and provide actionable recommendations based on the analysis.".into(), model: "default".into() },
                    depends_on: vec!["analyze".into()],
                    timeout_secs: 15, retry_count: 1, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn report_generation_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-report-gen".into(),
            name: "Report Generation".into(),
            description: "Daily/weekly report generation with research and formatting".into(),
            category: "reporting".into(),
            tags: vec!["report".into(), "generate".into(), "daily".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "collect_data".into(),
                    action: WorkflowAction::ToolCall { tool_id: "web_search".into(), params: serde_json::json!({"query": ""}) },
                    depends_on: vec![],
                    timeout_secs: 20, retry_count: 2, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "research".into(),
                    action: WorkflowAction::AgentCall { agent_id: "researcher".into(), input: "".into() },
                    depends_on: vec!["collect_data".into()],
                    timeout_secs: 30, retry_count: 1, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "analyze".into(),
                    action: WorkflowAction::LLMCall { system_prompt: "Analyze collected data and extract key insights.".into(), model: "default".into() },
                    depends_on: vec!["research".into()],
                    timeout_secs: 20, retry_count: 1, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "draft".into(),
                    action: WorkflowAction::LLMCall { system_prompt: "Draft a well-structured report with sections: Executive Summary, Findings, Analysis, Recommendations.".into(), model: "default".into() },
                    depends_on: vec!["analyze".into()],
                    timeout_secs: 30, retry_count: 1, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "review".into(),
                    action: WorkflowAction::HumanApproval { message: "Please review the generated report".into() },
                    depends_on: vec!["draft".into()],
                    timeout_secs: 300, retry_count: 0, approval_required: true,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "deliver".into(),
                    action: WorkflowAction::Notification { channel: "email".into(), message: "Report ready".into() },
                    depends_on: vec!["review".into()],
                    timeout_secs: 10, retry_count: 2, approval_required: false,
                    input_mapping: HashMap::new(), output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn news_monitor_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-news-monitor".into(),
            name: "News Monitor".into(),
            description: "Continuous news monitoring and alerting workflow".into(),
            category: "monitoring".into(),
            tags: vec!["news".into(), "monitor".into(), "alert".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 45,
            steps: vec![
                WorkflowStep {
                    id: "fetch_sources".into(),
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
                    id: "filter_relevant".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt:
                            "Filter and rank news items by relevance to the user's interests."
                                .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["fetch_sources".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "summarize".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Summarize each important news item in 2-3 sentences."
                            .into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["filter_relevant".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "assess_impact".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Assess the potential impact of each news item.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["summarize".into()],
                    timeout_secs: 10,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "alert".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "".into(),
                    },
                    depends_on: vec!["assess_impact".into()],
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
