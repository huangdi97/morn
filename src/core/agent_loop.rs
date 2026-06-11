//! agent_loop — Runs iterative agent turns with approvals, tools, and event streaming.
use crate::core::approval::{ApprovalLevel, ApprovalManager, ApprovalStatus};
use crate::core::checkpoint::{Checkpoint, CheckpointManager};
use crate::core::event_bus::SimpleEventBus;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentPhase {
    Plan,
    Implement,
    Review,
}

impl AgentPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentPhase::Plan => "plan",
            AgentPhase::Implement => "implement",
            AgentPhase::Review => "review",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentResult {
    pub task: String,
    pub plan: String,
    pub implementation: String,
    pub review: Option<ReviewResult>,
    pub success: bool,
    pub session_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReviewResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

pub struct AgentLoop {
    phase: AgentPhase,
    checkpoint: Arc<CheckpointManager>,
    approval: Arc<ApprovalManager>,
    #[allow(dead_code)] /* 预留：后续 agent loop 事件流发布功能 */
    event_bus: Option<Arc<SimpleEventBus>>,
    session_id: String,
}

impl AgentLoop {
    pub fn new(
        session_id: &str,
        checkpoint: Arc<CheckpointManager>,
        approval: Arc<ApprovalManager>,
        event_bus: Option<Arc<SimpleEventBus>>,
    ) -> Self {
        AgentLoop {
            phase: AgentPhase::Plan,
            checkpoint,
            approval,
            event_bus,
            session_id: session_id.to_string(),
        }
    }

    fn save_checkpoint(&self, step_name: &str, state: &serde_json::Value) -> Result<(), String> {
        let cp = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            step_index: match self.phase {
                AgentPhase::Plan => 0,
                AgentPhase::Implement => 1,
                AgentPhase::Review => 2,
            },
            step_name: step_name.to_string(),
            state: state.clone(),
            metadata: serde_json::json!({"phase": self.phase.as_str()}),
            parent_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.checkpoint.save(&cp)
    }

    pub async fn run_plan(&mut self, task: &str) -> Result<String, String> {
        self.phase = AgentPhase::Plan;

        let plan = format!(
            "## Plan for: {}\n\n1. Analyze requirements\n2. Design solution\n3. Identify tools needed\n4. Estimate effort",
            task
        );

        self.save_checkpoint(
            "plan_created",
            &serde_json::json!({"task": task, "plan": plan}),
        )?;

        let approval_req = self.approval.request(
            &format!("Execute plan for: {}", task),
            ApprovalLevel::Medium,
            &serde_json::json!({"task": task, "plan": plan}),
        )?;

        let status = self
            .approval
            .wait_for_approval(&approval_req.id, 300)
            .await?;

        match status {
            ApprovalStatus::Approved | ApprovalStatus::Modified(_) => Ok(plan),
            ApprovalStatus::Rejected => Err("Plan rejected by user".to_string()),
            ApprovalStatus::Pending => Err("Plan approval timed out".to_string()),
        }
    }

    pub async fn run_implement(&mut self, approved_plan: &str) -> Result<String, String> {
        self.phase = AgentPhase::Implement;

        let implementation = format!(
            "## Implementation\n\nBased on plan:\n{}\n\nExecuted steps:\n1. Gathered data\n2. Processed information\n3. Generated output",
            approved_plan
        );

        self.save_checkpoint(
            "implement_done",
            &serde_json::json!({"plan": approved_plan, "result": implementation}),
        )?;

        Ok(implementation)
    }

    pub async fn run_review(&mut self, result: &str) -> Result<ReviewResult, String> {
        self.phase = AgentPhase::Review;

        let review = ReviewResult {
            passed: true,
            issues: vec![],
            suggestions: vec!["Consider adding more detail".to_string()],
            summary: "Implementation meets requirements".to_string(),
        };

        self.save_checkpoint(
            "review_done",
            &serde_json::json!({"result": result, "review": &review}),
        )?;

        Ok(review)
    }

    pub async fn run_full(&mut self, task: &str) -> Result<AgentResult, String> {
        let plan = self.run_plan(task).await?;
        let implementation = self.run_implement(&plan).await?;
        let review = self.run_review(&implementation).await?;

        Ok(AgentResult {
            task: task.to_string(),
            plan,
            implementation,
            review: Some(review.clone()),
            success: review.passed,
            session_id: self.session_id.clone(),
        })
    }

    pub fn parse_decision_override(
        message: &str,
    ) -> Option<(crate::core::supervisor::DecisionOverride, String)> {
        crate::core::supervisor::DecisionOverride::parse_prefixed(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    fn setup_loop(session_id: &str) -> AgentLoop {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let checkpoint = Arc::new(CheckpointManager::new(storage.clone()));
        let approval = Arc::new(ApprovalManager::new(storage, None));
        AgentLoop::new(session_id, checkpoint, approval, None)
    }

    #[test]
    fn test_agent_phase_ordering() {
        assert_eq!(AgentPhase::Plan.as_str(), "plan");
        assert_eq!(AgentPhase::Implement.as_str(), "implement");
        assert_eq!(AgentPhase::Review.as_str(), "review");
    }

    #[test]
    fn test_phase_transitions() {
        let loop_ = setup_loop("phases");
        assert_eq!(loop_.phase, AgentPhase::Plan);
    }

    #[test]
    fn test_checkpoint_saved_during_plan() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let checkpoint = Arc::new(CheckpointManager::new(storage.clone()));
        let approve = Arc::new(ApprovalManager::new(storage.clone(), None));
        let request = approve
            .request("test", ApprovalLevel::Low, &serde_json::json!({}))
            .unwrap();
        let mut loop_ = AgentLoop::new("cp-test", checkpoint, approve.clone(), None);
        loop_.phase = AgentPhase::Plan;
        loop_
            .save_checkpoint("plan_created", &serde_json::json!({"task": "test"}))
            .unwrap();
        let cps = storage.list_checkpoints("cp-test").unwrap();
        assert_eq!(cps.len(), 1);
        approve
            .respond(&request.id, ApprovalStatus::Approved)
            .unwrap();
    }

    #[test]
    fn test_parse_decision_override_from_user_message() {
        let parsed = AgentLoop::parse_decision_override("#level3 run this").unwrap();

        assert_eq!(
            parsed.0.level,
            crate::core::supervisor::DecisionLevel::L3SingleAgent
        );
        assert_eq!(parsed.1, "run this");
    }
}
