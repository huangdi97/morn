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

impl StudioTester {
    pub(super) fn measure_persona_injection() -> (TestStep, ()) {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_secs_f64(0.02));
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
        _knowledge_id: &str,
        _query: &str,
    ) -> (TestStep, String) {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_secs_f64(0.15));
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let result = format!("Knowledge retrieved for query: {}", _query);
        let step = TestStep {
            name: "knowledge_retrieval".into(),
            description: format!(
                "Knowledge retrieval: {} ({:.2}s)",
                _knowledge_id,
                duration_ms / 1000.0
            ),
            duration_ms,
            success: true,
            tokens_used: None,
            cost: None,
            input_preview: Some(_query.to_string()),
            output_preview: Some(result.clone()),
        };
        (step, result)
    }

    pub(super) fn measure_llm_call(model: &str, prompt: &str, tokens: u64) -> (TestStep, String) {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_secs_f64(1.2));
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let cost = tokens as f64 * 0.000002;
        let result = format!(
            "LLM response for prompt: {}",
            prompt.chars().take(50).collect::<String>()
        );
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
        std::thread::sleep(std::time::Duration::from_secs_f64(2.1));
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let result = format!("Tool {} executed with params: {}", tool_id, params);
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
        std::thread::sleep(std::time::Duration::from_secs_f64(3.0));
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let result = format!("Workflow {} completed with input: {}", workflow_id, input);
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
