//! 测试沙箱 — Agent/Tool/Workflow 的隔离测试执行环境
use std::time::Instant;

use super::{StudioTester, TestRunner, TestStep};

impl TestRunner {
    pub fn run_and_measure<F, T>(label: &str, description: &str, f: F) -> (TestStep, T)
    where
        F: FnOnce() -> Result<T, String>,
        T: Default + std::fmt::Debug,
    {
        let start = Instant::now();
        let result = f();
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(output) => (
                TestStep {
                    name: label.to_string(),
                    description: format!("{} ({:.2}s)", description, duration_ms / 1000.0),
                    duration_ms,
                    success: true,
                    tokens_used: None,
                    cost: None,
                    input_preview: None,
                    output_preview: Some(format!("{:?}", output)),
                },
                output,
            ),
            Err(e) => (
                TestStep {
                    name: label.to_string(),
                    description: format!(
                        "{} failed: {} ({:.2}s)",
                        description,
                        e,
                        duration_ms / 1000.0
                    ),
                    duration_ms,
                    success: false,
                    tokens_used: None,
                    cost: None,
                    input_preview: None,
                    output_preview: Some(e.clone()),
                },
                T::default(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_and_measure_returns_success_step_and_output() {
        let (step, value) =
            TestRunner::run_and_measure("sandbox", "start", || -> Result<String, String> {
                Ok("ready".into())
            });

        assert!(step.success);
        assert_eq!(value, "ready");
        assert_eq!(step.name, "sandbox");
        assert_eq!(step.output_preview, Some("\"ready\"".into()));
    }

    #[test]
    fn run_and_measure_returns_default_output_on_error() {
        let (step, value) =
            TestRunner::run_and_measure("sandbox", "cleanup", || -> Result<Vec<String>, String> {
                Err("denied".into())
            });

        assert!(!step.success);
        assert!(step.description.contains("denied"));
        assert!(value.is_empty());
    }

    #[test]
    fn rerun_step_uses_new_input_for_knowledge_step() {
        let step = StudioTester::new().rerun_step("agent", "agent-1", 1, "new query");

        assert_eq!(step.name, "knowledge_retrieval");
        assert_eq!(step.input_preview, Some("new query".into()));
        assert!(step.success);
    }
}

impl StudioTester {
    pub(super) fn measure_persona_injection() -> (TestStep, ()) {
        let start = Instant::now();
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let step = TestStep {
            name: "persona_injection".into(),
            description: format!("Enhance Prompt ({:.2}s)", duration_ms / 1000.0),
            duration_ms,
            success: true,
            tokens_used: None,
            cost: None,
            input_preview: None,
            output_preview: None,
        };
        (step, ())
    }

    pub(super) fn measure_knowledge_retrieval(
        knowledge_id: &str,
        query: &str,
    ) -> (TestStep, String) {
        let start = Instant::now();
        let result = format!("Knowledge retrieved for query: {}", query);
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let step = TestStep {
            name: "knowledge_retrieval".into(),
            description: format!(
                "Knowledge retrieval: {} ({:.2}s)",
                knowledge_id,
                duration_ms / 1000.0
            ),
            duration_ms,
            success: true,
            tokens_used: None,
            cost: None,
            input_preview: Some(query.to_string()),
            output_preview: Some(result.clone()),
        };
        (step, result)
    }

    pub(super) fn measure_llm_call(model: &str, prompt: &str, tokens: u64) -> (TestStep, String) {
        let start = Instant::now();
        let cost = tokens as f64 * 0.000002;
        let result = format!(
            "LLM response for prompt: {}",
            prompt.chars().take(50).collect::<String>()
        );
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let step = TestStep {
            name: "llm_call".into(),
            description: format!(
                "LLM call: {} ({:.2}s, {} tokens)",
                model,
                duration_ms / 1000.0,
                tokens
            ),
            duration_ms,
            success: true,
            tokens_used: Some(tokens),
            cost: Some(cost),
            input_preview: Some(prompt.to_string()),
            output_preview: Some(result.clone()),
        };
        (step, result)
    }

    pub(super) fn measure_tool_execution(tool_id: &str, params: &str) -> (TestStep, String) {
        let start = Instant::now();
        let result = format!("Tool {} executed with params: {}", tool_id, params);
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let step = TestStep {
            name: "tool_execution".into(),
            description: format!("Tool execution: {} ({:.2}s)", tool_id, duration_ms / 1000.0),
            duration_ms,
            success: true,
            tokens_used: None,
            cost: None,
            input_preview: Some(params.to_string()),
            output_preview: Some(result.clone()),
        };
        (step, result)
    }

    pub(super) fn measure_workflow_execution(workflow_id: &str, input: &str) -> (TestStep, String) {
        let start = Instant::now();
        let result = format!("Workflow {} completed with input: {}", workflow_id, input);
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let step = TestStep {
            name: "workflow_execution".into(),
            description: format!(
                "Workflow execution: {} ({:.2}s)",
                workflow_id,
                duration_ms / 1000.0
            ),
            duration_ms,
            success: true,
            tokens_used: None,
            cost: None,
            input_preview: Some(input.to_string()),
            output_preview: Some(result.clone()),
        };
        (step, result)
    }

    pub fn rerun_step(
        &self,
        _component_type: &str,
        _component_id: &str,
        step_index: usize,
        new_input: &str,
    ) -> TestStep {
        match step_index {
            0 => Self::measure_persona_injection().0,
            1 => Self::measure_knowledge_retrieval("custom", new_input).0,
            2 => Self::measure_llm_call("deepseek", new_input, 890).0,
            3 => Self::measure_tool_execution("custom", new_input).0,
            _ => Self::measure_llm_call("deepseek", new_input, 890).0,
        }
    }
}
