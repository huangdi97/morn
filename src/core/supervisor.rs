//! COO 主管模块——负责用户意图的6级决策路由与任务拆解。
//!
//! 决策树结构（L1→L4 按复杂度递增，L5/L6 为特殊分支）：
//! - L1DirectAnswer: 简单问候/知识查询，直接回复
//! - L2SingleTool: 单工具操作，如搜索/计算/翻译
//! - L3SingleAgent: 单智能体分析任务
//! - L4Team: 多维度复杂任务，需多智能体协作
//! - L5Workflow: 标准化工作流，如生成报告/调研
//! - L6JumpToStudio: 用户意图涉及创建/修改组件，跳转工作室
//!
//! 核心职责: 接收用户输入 → 决策路由 → 生成 TaskPlan → 调度执行 → 记录反馈

use crate::core::event_bus::{SimpleEventBus, EVENT_SUPERVISOR_PLAN_CREATED, EVENT_TASK_COMPLETED};
use crate::core::storage::{DecisionRecord, DecisionRule, Storage, TaskRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLAgentDef {
    pub name: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskDef {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub params: Value,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub user_input: String,
    pub subtasks: Vec<SubTaskDef>,
    pub estimated_secs: u64,
    pub decision_level: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskResult {
    pub id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub subtask_results: Vec<SubTaskResult>,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurnRecord {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DecisionLevel {
    L1DirectAnswer,
    L2SingleTool,
    L3SingleAgent,
    L4Team,
    L5Workflow,
    L6JumpToStudio,
}

impl DecisionLevel {
    /// 将决策级别转为字符串标识。
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "direct_answer",
            DecisionLevel::L2SingleTool => "single_tool",
            DecisionLevel::L3SingleAgent => "single_agent",
            DecisionLevel::L4Team => "team",
            DecisionLevel::L5Workflow => "workflow",
            DecisionLevel::L6JumpToStudio => "jump_studio",
        }
    }

    /// 返回该决策级别的预估费用与耗时区间。
    pub fn cost_tier(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "¥0.001/0.5s",
            DecisionLevel::L2SingleTool => "¥0.003/1s",
            DecisionLevel::L3SingleAgent => "¥0.02/5s",
            DecisionLevel::L4Team => "¥0.05/15s",
            DecisionLevel::L5Workflow => "¥0.03/10s",
            DecisionLevel::L6JumpToStudio => "variable",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CooMode {
    Active,
    Safe,
    Auto,
}

impl CooMode {
    /// 返回 CooMode 的字符串标识。
    pub fn as_str(&self) -> &'static str {
        match self {
            CooMode::Active => "active",
            CooMode::Safe => "safe",
            CooMode::Auto => "auto",
        }
    }
}

pub struct Supervisor {
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
    history: Vec<TurnRecord>,
    max_history: usize,
    turn_count: u64,
    mode: CooMode,
    user_id: Option<String>,
    user_teams: Vec<String>,
}

const STOP_WORDS: &[&str] = &[
    "的", "了", "是", "在", "有", "和", "就", "不", "人", "都", "一", "个", "上", "也", "很", "到",
    "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他", "她", "它",
];

impl Supervisor {
    /// 创建 Supervisor 实例。
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        Supervisor {
            storage,
            event_bus,
            history: Vec::new(),
            max_history: 10,
            turn_count: 0,
            mode: CooMode::Active,
            user_id: None,
            user_teams: Vec::new(),
        }
    }

    /// 设置用户身份，返回自身（链式调用）。
    pub fn with_user(mut self, user_id: &str, teams: &[String]) -> Self {
        self.user_id = Some(user_id.to_string());
        self.user_teams = teams.to_vec();
        self
    }

    /// 设置当前用户身份。
    pub fn set_user(&mut self, user_id: &str, teams: &[String]) {
        self.user_id = Some(user_id.to_string());
        self.user_teams = teams.to_vec();
    }

    /// 返回当前用户 ID。
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// 返回当前用户所属团队列表。
    pub fn user_teams(&self) -> &[String] {
        &self.user_teams
    }

    /// 返回已处理的会话轮数。
    pub fn turn_count(&self) -> u64 {
        self.turn_count
    }
    /// 返回会话历史记录。
    pub fn history(&self) -> &[TurnRecord] {
        &self.history
    }
    /// 返回当前运行模式。
    pub fn mode(&self) -> &CooMode {
        &self.mode
    }
    /// 设置运行模式（Active/Safe/Auto）。
    pub fn set_mode(&mut self, mode: CooMode) {
        self.mode = mode;
    }

    /// 列出当前用户可用的智能体能力。
    pub fn list_available_agents<'a>(
        &self,
        registry: &'a crate::core::registry::Registry,
    ) -> Vec<&'a crate::core::registry::Capability> {
        registry.list_available(self.user_id.as_deref(), &self.user_teams)
    }

    /// 根据文本内容初步判定决策级别。
    pub fn decide_level(&self, text: &str) -> DecisionLevel {
        let text_lower = text.to_lowercase();

        let simple_indicators = [
            "hello",
            "hi ",
            "thanks",
            "bye",
            "who are you",
            "what is your name",
            "good morning",
        ];
        if simple_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower == s.trim())
        {
            return DecisionLevel::L1DirectAnswer;
        }

        let tool_indicators = [
            "search",
            "look up",
            "find ",
            "calculate",
            "compute",
            "convert",
            "translate",
            "what time",
            "what's the time",
        ];
        if tool_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L2SingleTool;
        }

        let studio_indicators = [
            "create an agent",
            "create a agent",
            "build an agent",
            "make an agent",
            "design a agent",
            "customize",
            "configure",
            "create workflow",
        ];
        if studio_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower.starts_with(s.trim()))
        {
            return DecisionLevel::L6JumpToStudio;
        }

        let workflow_indicators = [
            "report",
            "analysis",
            "research",
            "investigate",
            "compare",
            "plan",
            "strategy",
            "create a",
        ];
        if workflow_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L5Workflow;
        }

        let team_indicators = [
            "complex",
            "multi-step",
            "multiple",
            "various",
            "comprehensive",
            "full",
            "end-to-end",
        ];
        if team_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L4Team;
        }

        DecisionLevel::L3SingleAgent
    }

    /// 构建包含历史上下文的提示词。
    pub fn build_context(&self, current_input: &str) -> String {
        let mut context = String::from(
            "[System]\nYou are Morn, a helpful AI assistant.\n\n[Conversation History]\n",
        );

        let start = if self.history.len() > self.max_history {
            self.history.len() - self.max_history
        } else {
            0
        };

        for turn in &self.history[start..] {
            let role = if turn.role == "user" {
                "User"
            } else {
                "Assistant"
            };
            context.push_str(&format!("{}: {}\n", role, turn.content));
        }

        context.push_str(&format!(
            "\n[Current]\nUser: {}\nAssistant:\n",
            current_input
        ));
        context
    }

    /// 记录一条会话轮次。
    pub fn record_turn(&mut self, role: &str, content: &str) {
        self.history.push(TurnRecord {
            role: role.to_string(),
            content: content.to_string(),
        });
        if self.history.len() > self.max_history * 2 {
            self.history.remove(0);
        }
    }

    /// 判定决策级别并给出理由。
    pub fn decide(&self, text: &str) -> (DecisionLevel, String) {
        let level = self.decide_level(text);
        let reasoning = match level {
            DecisionLevel::L1DirectAnswer => "Simple greeting or knowledge query".into(),
            DecisionLevel::L2SingleTool => "Single tool operation needed".into(),
            DecisionLevel::L3SingleAgent => "Requires single agent analysis".into(),
            DecisionLevel::L4Team => "Complex multi-dimensional task".into(),
            DecisionLevel::L5Workflow => "Standard workflow available".into(),
            DecisionLevel::L6JumpToStudio => "User wants to create/modify components".into(),
        };
        (level, reasoning)
    }

    /// 执行任务计划，包含事件发布、存储记录、安全模式审批。
    pub fn execute_plan(
        &mut self,
        plan: &TaskPlan,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<TaskResult, String> {
        self.turn_count += 1;

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_SUPERVISOR_PLAN_CREATED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "user_input": plan.user_input,
                    "decision_level": plan.decision_level,
                    "mode": self.mode.as_str(),
                }),
            );
        }

        if let Some(ref storage) = self.storage {
            let task_record = TaskRecord {
                id: plan.task_id.clone(),
                user_input: plan.user_input.clone(),
                plan_json: serde_json::to_string(plan).unwrap_or_default(),
                status: "executing".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            };
            let _ = storage.insert_task(&task_record);

            let decision = DecisionRecord {
                id: format!("dec-{}", uuid::Uuid::new_v4()),
                task_id: plan.task_id.clone(),
                decision_level: plan.decision_level.clone(),
                action: format!("execute with {} subtasks", plan.subtasks.len()),
                context_json: Some(serde_json::json!({"mode": self.mode.as_str(), "estimated_secs": plan.estimated_secs}).to_string()),
                approved: self.mode != CooMode::Safe,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = storage.insert_decision(&decision);
        }

        if self.mode == CooMode::Safe {
            let preview = self.build_context(&plan.user_input);
            eprintln!("[COO Safe Mode] Plan requires approval:");
            eprintln!("  Level: {}", plan.decision_level);
            eprintln!("  Subtasks: {}", plan.subtasks.len());
            eprintln!("  Estimated: {}s", plan.estimated_secs);
            eprintln!("  Preview: {}...", &preview[..preview.len().min(200)]);
        }

        let context = self.build_context(&plan.user_input);

        let response = chat_fn(&context, "You are Morn, a helpful AI assistant.")?;

        self.record_turn("user", &plan.user_input);
        self.record_turn("assistant", &response);

        let result = TaskResult {
            task_id: plan.task_id.clone(),
            subtask_results: vec![SubTaskResult {
                id: "main".to_string(),
                success: true,
                output: response.clone(),
                error: None,
            }],
            summary: response.clone(),
        };

        if let Some(ref storage) = self.storage {
            let _ = storage.update_task_status(&plan.task_id, "completed");
        }

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_TASK_COMPLETED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "summary": result.summary,
                }),
            );
        }

        Ok(result)
    }

    /// 执行一次聊天：自动判定级别、生成计划、执行并返回结果。
    pub fn execute_chat(
        &mut self,
        input: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<String, String> {
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let (level, _reasoning) = self.decide(input);

        let plan = TaskPlan {
            task_id: task_id.clone(),
            user_input: input.to_string(),
            subtasks: vec![SubTaskDef {
                id: "main".to_string(),
                agent_id: "chat-agent".to_string(),
                action: "chat".to_string(),
                params: serde_json::json!({"input": input}),
                depends_on: vec![],
            }],
            estimated_secs: match level {
                DecisionLevel::L1DirectAnswer => 1,
                DecisionLevel::L2SingleTool => 3,
                DecisionLevel::L3SingleAgent => 10,
                DecisionLevel::L4Team => 30,
                DecisionLevel::L5Workflow => 20,
                DecisionLevel::L6JumpToStudio => 60,
            },
            decision_level: level.as_str().to_string(),
        };

        let result = self.execute_plan(&plan, chat_fn)?;
        Ok(result.summary)
    }

    /// 根据自然语言描述创建智能体定义。
    pub fn create_agent_from_nl(
        &self,
        nl: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<NLAgentDef, String> {
        let system_prompt = "You are an agent configuration assistant. Analyze the user's natural language description and return a JSON object with the agent definition. Only return valid JSON, no markdown, no explanation.";

        let prompt = format!(
            r#"User wants to create an agent. Analyze this description:
{}
Available personas: assistant, analyst, researcher, writer, coder, translator, reviewer
Available models: deepseek-chat, deepseek-reasoner
Available tools: web_search, read_file, write_file, exec_python, calc, get_time, get_kline, calc_macd, chart
Available knowledge: docs, glossary, data_sources
Available skills: summarization, translation, code_review, grammar_check, format, style, proofread, report_generation, debug, test

Return a JSON object with exactly these fields (all strings or string arrays):
{{
  "name": "short agent name (2-5 words)",
  "persona": "most appropriate persona from the list above",
  "model": "deepseek-chat",
  "tools": ["list", "of", "tool", "names"],
  "knowledge": ["list", "of", "knowledge", "sources"],
  "skills": ["list", "of", "skills"]
}}
Select tools, knowledge, and skills that best match the user's described use case."#,
            nl
        );

        let response = chat_fn(&prompt, system_prompt)?;

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str::<NLAgentDef>(cleaned).map_err(|e| {
            format!(
                "Failed to parse LLM response as AgentDef: {}. Raw: {}",
                e, cleaned
            )
        })
    }

    /// 基于已存储的规则表判定决策级别。
    pub fn decide_with_rules(&self, text: &str) -> (DecisionLevel, String) {
        if let Some(ref storage) = self.storage {
            let user_id = self.user_id.as_deref().unwrap_or("default");
            let keywords = Self::extract_keywords(text);
            for kw in &keywords {
                let rules = storage.get_decision_rules(user_id, kw).unwrap_or_default();
                if let Some(rule) = rules.first() {
                    if let Some(ref storage) = self.storage {
                        let _ = storage.increment_rule_hit(rule.id.unwrap_or(0));
                    }
                    if rule.auto_execute {
                        return (
                            DecisionLevel::L1DirectAnswer,
                            format!("Rule auto-execute matched keyword '{}'", kw),
                        );
                    }
                    let level = match rule.level.as_str() {
                        "direct_answer" => DecisionLevel::L1DirectAnswer,
                        "single_tool" => DecisionLevel::L2SingleTool,
                        "single_agent" => DecisionLevel::L3SingleAgent,
                        "team" => DecisionLevel::L4Team,
                        "workflow" => DecisionLevel::L5Workflow,
                        "jump_studio" => DecisionLevel::L6JumpToStudio,
                        _ => DecisionLevel::L3SingleAgent,
                    };
                    return (
                        level,
                        format!("Rule matched keyword '{}' with level {}", kw, rule.level),
                    );
                }
            }
        }
        self.decide(text)
    }

    /// 从用户反馈中学习，更新或创建决策规则。
    pub fn learn_from_feedback(&mut self, user_input: &str, approved: bool) -> Result<(), String> {
        let user_id = self.user_id.as_deref().unwrap_or("default").to_string();
        let keywords = Self::extract_keywords(user_input);
        if keywords.is_empty() {
            return Ok(());
        }
        let keyword = keywords[0].clone();
        let level = self.decide_level(user_input).as_str().to_string();

        if let Some(ref storage) = self.storage {
            let existing = storage
                .get_decision_rules(&user_id, &keyword)
                .unwrap_or_default();
            if let Some(rule) = existing.first() {
                let change = if approved { -10.0 } else { 15.0 };
                if let Some(rule_id) = rule.id {
                    storage.adjust_rule_threshold(rule_id, change)?;
                }
            } else {
                let rule = DecisionRule {
                    id: None,
                    user_id: user_id.clone(),
                    keyword: keyword.clone(),
                    level,
                    trust_threshold: if approved { 50.0 } else { 75.0 },
                    auto_execute: approved,
                    source: "learned".to_string(),
                    hit_count: 1,
                    last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                    created_at: None,
                };
                storage.upsert_decision_rule(&rule)?;
            }
        }
        Ok(())
    }

    fn extract_keywords(text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let raw: Vec<&str> = text_lower.split_whitespace().collect();
        let mut keywords: Vec<String> = Vec::new();
        let stop_set: std::collections::HashSet<&str> = STOP_WORDS.iter().copied().collect();
        for word in raw {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if !cleaned.is_empty() && !stop_set.contains(cleaned.as_str()) && cleaned.len() >= 2 {
                keywords.push(cleaned);
            }
        }
        keywords.sort();
        keywords.dedup();
        keywords
    }

    /// 清空会话历史和轮次计数。
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.turn_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_build_context() {
        let supervisor = Supervisor::new(None, None);
        let context = supervisor.build_context("hello");
        assert!(context.contains("You are Morn"));
        assert!(context.contains("[Current]"));
        assert!(context.contains("hello"));
    }

    #[test]
    fn test_supervisor_record_and_context() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.record_turn("user", "hi");
        supervisor.record_turn("assistant", "hello!");
        let context = supervisor.build_context("how are you?");
        assert!(context.contains("hi"));
        assert!(context.contains("hello!"));
        assert!(context.contains("how are you?"));
    }

    #[test]
    fn test_decide_level_simple() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("hello"),
            DecisionLevel::L1DirectAnswer
        );
        assert_eq!(
            supervisor.decide_level("thanks"),
            DecisionLevel::L1DirectAnswer
        );
    }

    #[test]
    fn test_decide_level_tool() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("search for AI news"),
            DecisionLevel::L2SingleTool
        );
        assert_eq!(
            supervisor.decide_level("calculate 2+2"),
            DecisionLevel::L2SingleTool
        );
    }

    #[test]
    fn test_decide_level_workflow() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("create a report"),
            DecisionLevel::L5Workflow
        );
        assert_eq!(
            supervisor.decide_level("analysis"),
            DecisionLevel::L5Workflow
        );
    }

    #[test]
    fn test_decide_level_studio() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("create an agent"),
            DecisionLevel::L6JumpToStudio
        );
    }

    #[test]
    fn test_decide_level_default() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("tell me about quantum physics"),
            DecisionLevel::L3SingleAgent
        );
    }

    #[test]
    fn test_coo_mode() {
        let mut supervisor = Supervisor::new(None, None);
        assert_eq!(*supervisor.mode(), CooMode::Active);
        supervisor.set_mode(CooMode::Safe);
        assert_eq!(*supervisor.mode(), CooMode::Safe);
    }

    #[test]
    fn test_decide_reasoning() {
        let supervisor = Supervisor::new(None, None);
        let (level, _reasoning) = supervisor.decide("complex multi-step task");
        assert_eq!(level, DecisionLevel::L4Team);
    }
}
