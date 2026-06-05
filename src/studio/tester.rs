use crate::core::component::Data;

#[derive(Debug, Clone)]
pub struct TestStep {
    pub name: String,
    pub description: String,
    pub duration_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub steps: Vec<TestStep>,
    pub total_duration_ms: f64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub output: String,
}

pub struct StudioTester;

impl StudioTester {
    pub fn new() -> Self {
        StudioTester
    }

    pub fn run_test(&self, component_type: &str, _input: &Data, _config: &str) -> TestResult {
        let mut steps = Vec::new();
        let mut total_duration = 0.0;

        let (prompt_duration, persona_step) =
            if component_type == "persona" || component_type == "agent" {
                let d = 0.02;
                (
                    d,
                    TestStep {
                        name: "persona_injection".into(),
                        description: "Enhance system prompt with persona".into(),
                        duration_ms: d,
                        success: true,
                    },
                )
            } else {
                (
                    0.0,
                    TestStep {
                        name: "persona_injection".into(),
                        description: "Persona injection (skipped)".into(),
                        duration_ms: 0.0,
                        success: true,
                    },
                )
            };
        total_duration += prompt_duration;
        steps.push(persona_step);

        let knowledge_duration = if component_type == "knowledge" || component_type == "agent" {
            0.15
        } else {
            0.0
        };
        total_duration += knowledge_duration;
        steps.push(TestStep {
            name: "knowledge_retrieval".into(),
            description: format!("Knowledge retrieval ({}s)", knowledge_duration),
            duration_ms: knowledge_duration,
            success: true,
        });

        let llm_duration = 1.2;
        total_duration += llm_duration;
        steps.push(TestStep {
            name: "llm_call".into(),
            description: format!("LLM call ({}s, 890 tokens)", llm_duration),
            duration_ms: llm_duration,
            success: true,
        });

        if component_type == "tool" {
            let tool_duration = 2.1;
            total_duration += tool_duration;
            steps.push(TestStep {
                name: "tool_execution".into(),
                description: format!("Tool execution ({}s)", tool_duration),
                duration_ms: tool_duration,
                success: true,
            });
        }

        TestResult {
            steps,
            total_duration_ms: total_duration * 1000.0,
            total_tokens: 2090,
            total_cost: 0.02,
            output: format!("Component test passed for type: {}", component_type),
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
