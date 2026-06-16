//! tester — Provides studio test execution and result collection.
use crate::core::error::MornError;
use std::sync::Arc;

use crate::core::component::Data;
use crate::core::registry::Registry;
use crate::core::storage::Storage;

mod reports;
mod sandbox;

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
    #[allow(dead_code)] /* 预留：测试运行时 registry 注入 */ registry: Arc<Registry>,
    #[allow(dead_code)] /* 预留：测试结果落库 */ storage: Arc<Storage>,
}

impl TestRunner {
    pub fn new(registry: Arc<Registry>, storage: Arc<Storage>) -> Self {
        TestRunner { registry, storage }
    }
}

pub struct StudioTester;

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
}

impl Default for StudioTester {
    fn default() -> Self {
        Self::new()
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
            TestRunner::run_and_measure("test", "test fn", || -> Result<i32, MornError> { Ok(42) });
        assert!(step.success);
        assert_eq!(val, 42);
        assert!(step.duration_ms >= 0.0);
    }

    #[test]
    fn test_run_and_measure_err() {
        let (step, val) =
            TestRunner::run_and_measure("test", "test fn", || -> Result<i32, MornError> {
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
