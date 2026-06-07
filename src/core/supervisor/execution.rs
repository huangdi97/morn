//! execution — Supervises execution plans and emits task lifecycle events.
use crate::core::event_bus::{EVENT_SUPERVISOR_PLAN_CREATED, EVENT_TASK_COMPLETED};
use crate::core::storage::{DecisionRecord, DecisionRule, TaskRecord};

use super::{
    CooMode, DecisionLevel, NLAgentDef, SubTaskDef, SubTaskResult, Supervisor, TaskPlan, TaskResult,
};

impl Supervisor {
    /// Executes a task plan with the provided chat function and returns the completed task result.
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

    /// Builds and executes a single-turn chat plan for the input, returning the response summary.
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

    /// Converts a natural-language agent request into an agent definition using the provided chat function.
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

    /// Learns a decision rule from user feedback and returns success when storage updates complete.
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
}
