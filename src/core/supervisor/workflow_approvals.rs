//! workflow_approvals — Tracks pending workflow approval gates in Supervisor.
use crate::core::approval::{ApprovalStatus, WorkflowApproval};

use super::Supervisor;

impl Supervisor {
    pub fn register_workflow_approval(&mut self, approval: WorkflowApproval) {
        if self.workflow_approvals.iter().any(|existing| {
            existing.workflow_id == approval.workflow_id && existing.step_id == approval.step_id
        }) {
            return;
        }
        self.workflow_approvals.push(approval);
    }

    pub fn pending_approvals(&self) -> Vec<WorkflowApproval> {
        self.workflow_approvals
            .iter()
            .filter(|approval| approval.status == ApprovalStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn approve_workflow_step(
        &mut self,
        workflow_id: &str,
        step_id: &str,
        approved: bool,
        comment: Option<String>,
    ) -> Result<WorkflowApproval, String> {
        let approval = self
            .workflow_approvals
            .iter_mut()
            .find(|approval| approval.workflow_id == workflow_id && approval.step_id == step_id)
            .ok_or_else(|| format!("Workflow approval not found: {}/{}", workflow_id, step_id))?;

        approval.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        approval.comment = comment;
        Ok(approval.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approval() -> WorkflowApproval {
        WorkflowApproval {
            workflow_id: "wf-1".to_string(),
            step_id: "step-1".to_string(),
            action: "deploy".to_string(),
            status: ApprovalStatus::Pending,
            assigned_to: Some("owner".to_string()),
            comment: None,
        }
    }

    #[test]
    fn supervisor_lists_pending_workflow_approvals() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.register_workflow_approval(approval());

        let pending = supervisor.pending_approvals();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].step_id, "step-1");
    }

    #[test]
    fn supervisor_approves_workflow_step() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.register_workflow_approval(approval());

        let updated = supervisor
            .approve_workflow_step("wf-1", "step-1", true, Some("go".to_string()))
            .unwrap();

        assert_eq!(updated.status, ApprovalStatus::Approved);
        assert!(supervisor.pending_approvals().is_empty());
    }
}
