//! reports — Builds studio test reports from executed test results.
use super::{StudioTester, TestResult};

impl StudioTester {
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
    use crate::studio::tester::TestStep;

    fn sample_result() -> TestResult {
        TestResult {
            steps: vec![
                TestStep {
                    name: "prepare".into(),
                    description: "load config".into(),
                    duration_ms: 250.0,
                    success: true,
                    tokens_used: None,
                    cost: None,
                    input_preview: None,
                    output_preview: None,
                },
                TestStep {
                    name: "execute".into(),
                    description: "run agent".into(),
                    duration_ms: 750.0,
                    success: true,
                    tokens_used: Some(42),
                    cost: Some(0.01),
                    input_preview: Some("input".into()),
                    output_preview: Some("output".into()),
                },
            ],
            total_duration_ms: 1000.0,
            total_tokens: 42,
            total_cost: 0.01,
            output: "done".into(),
        }
    }

    #[test]
    fn format_result_lists_each_step() {
        let formatted = StudioTester::new().format_result(&sample_result());

        assert!(formatted.contains("[1] prepare: load config"));
        assert!(formatted.contains("[2] execute: run agent"));
    }

    #[test]
    fn format_result_converts_total_duration_to_seconds() {
        let formatted = StudioTester::new().format_result(&sample_result());

        assert!(formatted.contains("Total: 1.00s"));
    }

    #[test]
    fn format_result_includes_token_and_cost_totals() {
        let formatted = StudioTester::new().format_result(&sample_result());

        assert!(formatted.contains("Tokens: 42"));
        assert!(formatted.contains("Cost:"));
    }
}
