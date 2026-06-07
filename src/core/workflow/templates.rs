use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

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

    fn task_execution_template() -> WorkflowTemplate {
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

    fn deep_analysis_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-deep-analysis".into(),
            name: "Deep Analysis".into(),
            description: "Multi-source research and deep analysis workflow".into(),
            category: "research".into(),
            tags: vec!["analysis".into(), "research".into(), "data".into()],
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

    fn news_monitor_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-news-monitor".into(),
            name: "News Monitor".into(),
            description: "Continuous news monitoring and alerting workflow".into(),
            category: "monitoring".into(),
            tags: vec!["news".into(), "monitor".into(), "alert".into()],
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

    fn report_generation_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-report-gen".into(),
            name: "Report Generation".into(),
            description: "Daily/weekly report generation with research and formatting".into(),
            category: "reporting".into(),
            tags: vec!["report".into(), "generate".into(), "daily".into()],
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

    fn code_delivery_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-code-delivery".into(),
            name: "Code Delivery".into(),
            description: "End-to-end software development workflow".into(),
            category: "development".into(),
            tags: vec!["code".into(), "development".into(), "delivery".into()],
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

    fn product_launch_template() -> WorkflowTemplate {
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

    fn decision_eval_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-decision-eval".into(),
            name: "Decision Evaluation".into(),
            description: "Multi-perspective decision evaluation for project proposals".into(),
            category: "strategy".into(),
            tags: vec!["decision".into(), "evaluation".into(), "strategy".into()],
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

    fn scheduled_inspection_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-scheduled-inspection".into(),
            name: "Scheduled Inspection".into(),
            description: "Regular system health and performance inspection".into(),
            category: "operations".into(),
            tags: vec!["ops".into(), "inspection".into(), "health".into()],
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
