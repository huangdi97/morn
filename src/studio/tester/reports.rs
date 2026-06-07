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
