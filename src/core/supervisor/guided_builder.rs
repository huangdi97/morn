//! guided_builder — Provides COO-guided workflow construction state.
use std::collections::HashMap;

use super::Supervisor;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GuidedBuildStep {
    Goal,
    Team,
    Tools,
    Safety,
    Preview,
    Launch,
    Complete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuidedBuildResponse {
    pub step: GuidedBuildStep,
    pub prompt: String,
    pub suggestions: Vec<String>,
    pub actions: Vec<String>,
    pub done: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuidedBuilder {
    goal: String,
    step: GuidedBuildStep,
    selections: HashMap<String, String>,
}

impl GuidedBuilder {
    pub fn new(goal: &str) -> Self {
        GuidedBuilder {
            goal: goal.to_string(),
            step: GuidedBuildStep::Goal,
            selections: HashMap::new(),
        }
    }

    pub fn current_step(&self) -> &GuidedBuildStep {
        &self.step
    }

    pub fn selections(&self) -> &HashMap<String, String> {
        &self.selections
    }

    pub fn advance(&mut self, input: &str) -> GuidedBuildResponse {
        self.record_selection(input);
        self.step = Self::next_step(&self.step);
        self.response()
    }

    pub fn response(&self) -> GuidedBuildResponse {
        let (prompt, suggestions) = match self.step {
            GuidedBuildStep::Goal => (
                format!("Define the workflow goal for '{}'", self.goal),
                vec![
                    "summarize the target outcome".to_string(),
                    "list success criteria".to_string(),
                ],
            ),
            GuidedBuildStep::Team => (
                "Choose the agent or team shape".to_string(),
                vec!["single specialist".to_string(), "review team".to_string()],
            ),
            GuidedBuildStep::Tools => (
                "Choose required tools".to_string(),
                vec![
                    "search".to_string(),
                    "code".to_string(),
                    "notify".to_string(),
                ],
            ),
            GuidedBuildStep::Safety => (
                "Choose safety gates".to_string(),
                vec![
                    "approval before publish".to_string(),
                    "audit log".to_string(),
                ],
            ),
            GuidedBuildStep::Preview => (
                "Review the assembled workflow".to_string(),
                vec!["revise".to_string(), "launch".to_string()],
            ),
            GuidedBuildStep::Launch => (
                "Launch the workflow".to_string(),
                vec!["start now".to_string(), "schedule later".to_string()],
            ),
            GuidedBuildStep::Complete => ("Workflow build complete".to_string(), vec![]),
        };

        GuidedBuildResponse {
            step: self.step.clone(),
            prompt,
            suggestions,
            actions: Self::default_actions(),
            done: self.step == GuidedBuildStep::Complete,
        }
    }

    fn default_actions() -> Vec<String> {
        vec![
            "直接保存".to_string(),
            "修改".to_string(),
            "预览详情".to_string(),
        ]
    }

    fn record_selection(&mut self, input: &str) {
        let key = format!("{:?}", self.step).to_lowercase();
        self.selections.insert(key, input.to_string());
    }

    fn next_step(step: &GuidedBuildStep) -> GuidedBuildStep {
        match step {
            GuidedBuildStep::Goal => GuidedBuildStep::Team,
            GuidedBuildStep::Team => GuidedBuildStep::Tools,
            GuidedBuildStep::Tools => GuidedBuildStep::Safety,
            GuidedBuildStep::Safety => GuidedBuildStep::Preview,
            GuidedBuildStep::Preview => GuidedBuildStep::Launch,
            GuidedBuildStep::Launch => GuidedBuildStep::Complete,
            GuidedBuildStep::Complete => GuidedBuildStep::Complete,
        }
    }
}

impl Supervisor {
    pub fn start_guided_build(&mut self, goal: &str) -> GuidedBuildResponse {
        let builder = GuidedBuilder::new(goal);
        let response = builder.response();
        self.guided_builder = Some(builder);
        response
    }

    pub fn guided_step(&mut self, input: &str) -> Result<GuidedBuildResponse, String> {
        let builder = self
            .guided_builder
            .as_mut()
            .ok_or_else(|| "Guided build has not started".to_string())?;
        Ok(builder.advance(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guided_builder_advances_steps() {
        let mut builder = GuidedBuilder::new("ship release");

        assert_eq!(builder.current_step(), &GuidedBuildStep::Goal);
        let response = builder.advance("release safely");

        assert_eq!(response.step, GuidedBuildStep::Team);
        assert_eq!(response.actions, vec!["直接保存", "修改", "预览详情"]);
        assert_eq!(
            builder.selections().get("goal").map(String::as_str),
            Some("release safely")
        );
    }

    #[test]
    fn supervisor_runs_guided_build_session() {
        let mut supervisor = Supervisor::new(None, None);

        let initial = supervisor.start_guided_build("publish report");
        let next = supervisor.guided_step("clear report").unwrap();

        assert_eq!(initial.step, GuidedBuildStep::Goal);
        assert_eq!(next.step, GuidedBuildStep::Team);
    }
}
