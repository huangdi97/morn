use std::sync::Arc;
use std::time::Instant;

use crate::core::component::Data;
use crate::core::registry::Registry;
use crate::core::storage::Storage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestStep {
    pub name: String,
    pub description: String,
    pub duration_ms: f64,
    pub success: bool,
    #[serde(default)]
    pub tokens_used: Option<u64>,
    #[serde(default)]
    pub cost: Option<f64>,
    #[serde(default)]
    pub input_preview: Option<String>,
    #[serde(default)]
    pub output_preview: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResult {
    pub steps: Vec<TestStep>,
    pub total_duration_ms: f64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub output: String,
}

pub struct TestRunner {
    #[allow(dead_code)]
    registry: Arc<Registry>,
    #[allow(dead_code)]
    storage: Arc<Storage>,
}

impl TestRunner {
    pub fn new(registry: Arc<Registry>, storage: Arc<Storage>) -> Self {
        TestRunner { registry, storage }
    }

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

pub struct StudioTester;

impl Default for StudioTester {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioTester {
    pub fn new() -> Self {
        StudioTester
    }

    pub fn run_test(
        &self,
        component_type: &str,
        component_id: &str,
        input: &Data,
        _config: &str,
    ) -> TestResult {
        let mut steps = Vec::new();
        let mut total_duration = 0.0;
        let mut total_tokens: u64 = 0;
        let mut total_cost: f64 = 0.0;

        let input_text = match &input.content {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        let final_output = match component_type {
            "agent" => {
                let (step, _) = Self::measure_persona_injection();
                total_duration += step.duration_ms;
                steps.push(step);

                let (step, _) = Self::measure_knowledge_retrieval(component_id, &input_text);
                total_duration += step.duration_ms;
                steps.push(step);

                let (mut step, output) = Self::measure_llm_call("deepseek", &input_text, 890);
                total_tokens += step.tokens_used.unwrap_or(0);
                total_cost += step.cost.unwrap_or(0.0);
                step.input_preview = Some(input_text.clone());
                step.output_preview = Some(output.clone());
                total_duration += step.duration_ms;
                steps.push(step);
                output
            }
            "tool" => {
                let (step, output) = Self::measure_tool_execution(component_id, &input_text);
                total_duration += step.duration_ms;
                steps.push(step);
                output
            }
            "workflow" => {
                let (step, output) = Self::measure_workflow_execution(component_id, &input_text);
                total_duration += step.duration_ms;
                steps.push(step);
                output
            }
            _ => {
                let (step, _) = Self::measure_persona_injection();
                total_duration += step.duration_ms;
                steps.push(step);

                let (mut step, output) = Self::measure_llm_call("deepseek", &input_text, 890);
                total_tokens += step.tokens_used.unwrap_or(0);
                total_cost += step.cost.unwrap_or(0.0);
                step.input_preview = Some(input_text.clone());
                step.output_preview = Some(output.clone());
                total_duration += step.duration_ms;
                steps.push(step);
                output
            }
        };

        TestResult {
            steps,
            total_duration_ms: total_duration,
            total_tokens,
            total_cost,
            output: final_output,
        }
    }

    fn measure_persona_injection() -> (TestStep, ()) {
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

    fn measure_knowledge_retrieval(_knowledge_id: &str, _query: &str) -> (TestStep, String) {
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

    fn measure_llm_call(model: &str, prompt: &str, tokens: u64) -> (TestStep, String) {
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

    fn measure_tool_execution(tool_id: &str, params: &str) -> (TestStep, String) {
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

    fn measure_workflow_execution(workflow_id: &str, input: &str) -> (TestStep, String) {
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

    pub fn format_result(&self, result: &TestResult) -> String {
        let mut output = String::new();
        for (i, step) in result.steps.iter().enumerate() {
            output.push_str(&format!(
                "[{}] {}: {} ({:.2}s)\n",
                i + 1,
                step.name,
                step.description,
                step.duration_ms,
            ));
        }
        output.push_str(&format!(
            "---\nTotal: {:.2}s | Tokens: {} | Cost: ¥{:.2}\n",
            result.total_duration_ms / 1000.0,
            result.total_tokens,
            result.total_cost,
        ));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_step_new_fields() {
        let step = TestStep {
            name: "test".into(),
            description: "desc".into(),
            duration_ms: 1.0,
            success: true,
            tokens_used: Some(100),
            cost: Some(0.01),
            input_preview: Some("in".into()),
            output_preview: Some("out".into()),
        };
        assert_eq!(step.tokens_used, Some(100));
        assert_eq!(step.cost, Some(0.01));
    }

    #[test]
    fn test_test_step_default_serde() {
        let json = r#"{"name":"t","description":"d","duration_ms":1.0,"success":true}"#;
        let step: TestStep = serde_json::from_str(json).unwrap();
        assert_eq!(step.tokens_used, None);
        assert_eq!(step.cost, None);
    }

    #[test]
    fn test_run_and_measure_ok() {
        let (step, val) =
            TestRunner::run_and_measure("test", "test fn", || -> Result<i32, String> { Ok(42) });
        assert!(step.success);
        assert_eq!(val, 42);
        assert!(step.duration_ms >= 0.0);
    }

    #[test]
    fn test_run_and_measure_err() {
        let (step, val) =
            TestRunner::run_and_measure("test", "test fn", || -> Result<i32, String> {
                Err("oops".into())
            });
        assert!(!step.success);
        assert_eq!(val, 0);
    }

    #[test]
    fn test_studio_tester_run_test_agent() {
        let tester = StudioTester::new();
        let input = Data::text("test input");
        let result = tester.run_test("agent", "agent-1", &input, "");
        assert!(!result.steps.is_empty());
        assert!(result.total_duration_ms > 0.0);
    }

    #[test]
    fn test_studio_tester_run_test_tool() {
        let tester = StudioTester::new();
        let input = Data::text("tool params");
        let result = tester.run_test("tool", "get_kline", &input, "");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].name, "tool_execution");
    }

    #[test]
    fn test_studio_tester_run_test_workflow() {
        let tester = StudioTester::new();
        let input = Data::text("wf input");
        let result = tester.run_test("workflow", "wf-1", &input, "");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].name, "workflow_execution");
    }

    #[test]
    fn test_rerun_step() {
        let tester = StudioTester::new();
        let step = tester.rerun_step("agent", "agent-1", 2, "new input");
        assert!(step.success);
        assert_eq!(step.input_preview, Some("new input".into()));
    }

    #[test]
    fn test_format_result() {
        let tester = StudioTester::new();
        let result = TestResult {
            steps: vec![TestStep {
                name: "step1".into(),
                description: "desc".into(),
                duration_ms: 100.0,
                success: true,
                tokens_used: None,
                cost: None,
                input_preview: None,
                output_preview: None,
            }],
            total_duration_ms: 100.0,
            total_tokens: 500,
            total_cost: 0.01,
            output: "done".into(),
        };
        let formatted = tester.format_result(&result);
        assert!(formatted.contains("step1"));
        assert!(formatted.contains("Tokens: 500"));
    }
}
