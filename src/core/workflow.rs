use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VarType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
}

impl VarType {
    pub fn detect(value: &Value) -> Self {
        match value {
            Value::String(_) => VarType::String,
            Value::Number(_) => VarType::Number,
            Value::Bool(_) => VarType::Boolean,
            Value::Object(_) => VarType::Object,
            Value::Array(_) => VarType::Array,
            Value::Null => VarType::Null,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: VarType,
    pub value: Value,
    pub source_step: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableStore {
    variables: HashMap<String, Variable>,
}

impl VariableStore {
    pub fn new() -> Self {
        VariableStore {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, step_id: &str, name: &str, value: Value) -> Result<(), String> {
        let var_type = VarType::detect(&value);
        let key = format!("{}.{}", step_id, name);
        self.variables.insert(
            name.to_string(),
            Variable {
                name: name.to_string(),
                var_type,
                value: value.clone(),
                source_step: Some(step_id.to_string()),
            },
        );
        // Also store with step prefix for disambiguation
        self.variables.insert(
            key,
            Variable {
                name: name.to_string(),
                var_type: VarType::detect(&value),
                value,
                source_step: Some(step_id.to_string()),
            },
        );
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Variable, String> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    pub fn convert(&self, value: &Variable, target_type: VarType) -> Result<Variable, String> {
        let converted_value = match (&value.var_type, &target_type) {
            (VarType::String, VarType::Number) => {
                let s = value.value.as_str().ok_or("expected string")?;
                let n: f64 = s
                    .parse()
                    .map_err(|e| format!("cannot parse as number: {}", e))?;
                Value::Number(
                    serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
                )
            }
            (VarType::String, VarType::Boolean) => {
                let s = value.value.as_str().ok_or("expected string")?;
                Value::Bool(s == "true" || s == "1" || s == "yes")
            }
            (VarType::Number, VarType::String) => Value::String(value.value.to_string()),
            (VarType::Number, VarType::Boolean) => {
                let n = value.value.as_f64().unwrap_or(0.0);
                Value::Bool(n != 0.0)
            }
            (VarType::Boolean, VarType::String) => {
                let b = value.value.as_bool().unwrap_or(false);
                Value::String(b.to_string())
            }
            (VarType::Boolean, VarType::Number) => {
                let b = value.value.as_bool().unwrap_or(false);
                Value::Number(serde_json::Number::from(if b { 1 } else { 0 }))
            }
            _ => {
                if std::mem::discriminant(&value.var_type) == std::mem::discriminant(&target_type) {
                    value.value.clone()
                } else {
                    return Err(format!(
                        "Unsupported conversion from {:?} to {:?}",
                        value.var_type, target_type
                    ));
                }
            }
        };
        Ok(Variable {
            name: value.name.clone(),
            var_type: target_type,
            value: converted_value,
            source_step: value.source_step.clone(),
        })
    }

    pub fn all(&self) -> Vec<&Variable> {
        self.variables
            .iter()
            .filter(|(k, _)| !k.contains('.'))
            .map(|(_, v)| v)
            .collect()
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkflowAction {
    LLMCall {
        system_prompt: String,
        model: String,
    },
    ToolCall {
        tool_id: String,
        params: Value,
    },
    AgentCall {
        agent_id: String,
        input: String,
    },
    TeamCall {
        team_id: String,
        input: String,
    },
    SubWorkflow {
        workflow_id: String,
    },
    CodeExec {
        language: String,
        script: String,
    },
    KnowledgeQuery {
        knowledge_id: String,
        query: String,
    },
    HumanApproval {
        message: String,
    },
    HumanInput {
        question: String,
    },
    Notification {
        channel: String,
        message: String,
    },
    Condition {
        expression: String,
        true_branch: Vec<WorkflowStep>,
        false_branch: Vec<WorkflowStep>,
    },
    Loop {
        iterator: String,
        body: Vec<WorkflowStep>,
        max_iterations: u32,
    },
    Wait {
        duration_secs: u32,
    },
    Fork {
        branches: Vec<Vec<WorkflowStep>>,
    },
    Join,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub action: WorkflowAction,
    pub depends_on: Vec<String>,
    pub timeout_secs: u32,
    pub retry_count: u8,
    pub approval_required: bool,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub estimated_duration_secs: u64,
    pub category: String,
    pub tags: Vec<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_templates() {
        let templates = WorkflowTemplate::list_builtin();
        assert_eq!(templates.len(), 8);
    }

    #[test]
    fn test_get_template_by_id() {
        let template = WorkflowTemplate::get_by_id("workflow-task-execution");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Task Execution");
    }

    #[test]
    fn test_template_categories() {
        let templates = WorkflowTemplate::list_builtin();
        let categories: Vec<&str> = templates.iter().map(|t| t.category.as_str()).collect();
        assert!(categories.contains(&"general"));
        assert!(categories.contains(&"research"));
        assert!(categories.contains(&"development"));
        assert!(categories.contains(&"operations"));
    }

    #[test]
    fn test_task_execution_has_six_steps() {
        let t = WorkflowTemplate::get_by_id("workflow-task-execution").unwrap();
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn test_code_delivery_has_seven_steps() {
        let t = WorkflowTemplate::get_by_id("workflow-code-delivery").unwrap();
        assert_eq!(t.steps.len(), 7);
    }

    #[test]
    fn test_all_templates_have_steps() {
        for t in WorkflowTemplate::list_builtin() {
            assert!(!t.steps.is_empty(), "Template '{}' has no steps", t.id);
        }
    }

    #[test]
    fn test_workflow_action_serialization() {
        let action = WorkflowAction::ToolCall {
            tool_id: "web_search".into(),
            params: serde_json::json!({"q": "test"}),
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: WorkflowAction = serde_json::from_str(&json).unwrap();
        match deserialized {
            WorkflowAction::ToolCall { tool_id, .. } => assert_eq!(tool_id, "web_search"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_variable_store_set_and_get() {
        let mut store = VariableStore::new();
        store
            .set("step1", "result", serde_json::json!("hello"))
            .unwrap();
        let var = store.get("result").unwrap();
        assert_eq!(var.value, "hello");
        assert_eq!(var.source_step.unwrap(), "step1");
    }

    #[test]
    fn test_variable_store_get_missing() {
        let store = VariableStore::new();
        let result = store.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_store_type_detection() {
        let mut store = VariableStore::new();
        store.set("s1", "str", serde_json::json!("text")).unwrap();
        store.set("s1", "num", serde_json::json!(42)).unwrap();
        store.set("s1", "flag", serde_json::json!(true)).unwrap();
        store
            .set("s1", "arr", serde_json::json!([1, 2, 3]))
            .unwrap();
        store
            .set("s1", "obj", serde_json::json!({"k": "v"}))
            .unwrap();
        store.set("s1", "null", serde_json::json!(null)).unwrap();

        assert!(matches!(
            store.get("str").unwrap().var_type,
            VarType::String
        ));
        assert!(matches!(
            store.get("num").unwrap().var_type,
            VarType::Number
        ));
        assert!(matches!(
            store.get("flag").unwrap().var_type,
            VarType::Boolean
        ));
        assert!(matches!(store.get("arr").unwrap().var_type, VarType::Array));
        assert!(matches!(
            store.get("obj").unwrap().var_type,
            VarType::Object
        ));
        assert!(matches!(store.get("null").unwrap().var_type, VarType::Null));
    }

    #[test]
    fn test_variable_store_convert_string_to_number() {
        let var = Variable {
            name: "score".into(),
            var_type: VarType::String,
            value: serde_json::json!("95.5"),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::Number).unwrap();
        assert_eq!(converted.value, 95.5);
    }

    #[test]
    fn test_variable_store_convert_number_to_string() {
        let var = Variable {
            name: "count".into(),
            var_type: VarType::Number,
            value: serde_json::json!(42),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::String).unwrap();
        assert_eq!(converted.value, "42");
    }

    #[test]
    fn test_variable_store_convert_bool_to_string() {
        let var = Variable {
            name: "flag".into(),
            var_type: VarType::Boolean,
            value: serde_json::json!(true),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::String).unwrap();
        assert_eq!(converted.value, "true");
    }

    #[test]
    fn test_variable_store_all() {
        let mut store = VariableStore::new();
        store.set("a", "x", serde_json::json!(1)).unwrap();
        store.set("b", "y", serde_json::json!(2)).unwrap();
        assert_eq!(store.all().len(), 2);
    }

    #[test]
    fn test_variable_store_clear() {
        let mut store = VariableStore::new();
        store.set("a", "x", serde_json::json!(1)).unwrap();
        store.clear();
        assert!(store.get("x").is_err());
    }
}
