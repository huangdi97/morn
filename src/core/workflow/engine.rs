//! engine — Executes workflow control-flow nodes without replacing template storage.
use crate::core::approval::{ApprovalStatus, WorkflowApproval};
use crate::core::thread_pool::{TaskDef, TaskPool};
use crate::core::workflow::{WorkflowAction, WorkflowTemplate};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ControlFlowNode {
    Sequential(Vec<TaskDef>),
    Parallel(Vec<Vec<TaskDef>>),
    Conditional {
        condition: String,
        if_branch: Vec<TaskDef>,
        else_branch: Option<Vec<TaskDef>>,
    },
    Loop {
        max_iterations: u32,
        tasks: Vec<TaskDef>,
    },
}

pub struct WorkflowEngine {
    task_pool: TaskPool,
    context: HashMap<String, Value>,
    approvals: Vec<WorkflowApproval>,
}

impl WorkflowEngine {
    pub fn new(task_pool: TaskPool) -> Self {
        WorkflowEngine {
            task_pool,
            context: HashMap::new(),
            approvals: Vec::new(),
        }
    }

    pub fn context_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.context
    }

    pub fn pending_approvals(&self) -> Vec<WorkflowApproval> {
        self.approvals
            .iter()
            .filter(|approval| approval.status == ApprovalStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn approve_step(
        &mut self,
        workflow_id: &str,
        step_id: &str,
        approved: bool,
        comment: Option<String>,
    ) -> Result<WorkflowApproval, String> {
        let approval = self
            .approvals
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

    pub async fn execute_template(
        &mut self,
        template: &WorkflowTemplate,
    ) -> Result<Vec<String>, String> {
        self.execute_node(
            &template.id,
            &ControlFlowNode::Sequential(template.steps.clone()),
        )
        .await
    }

    pub async fn execute_node(
        &mut self,
        workflow_id: &str,
        node: &ControlFlowNode,
    ) -> Result<Vec<String>, String> {
        match node {
            ControlFlowNode::Sequential(tasks) => self.execute_tasks(workflow_id, tasks).await,
            ControlFlowNode::Parallel(branches) => {
                self.execute_parallel(workflow_id, branches).await
            }
            ControlFlowNode::Conditional {
                condition,
                if_branch,
                else_branch,
            } => {
                if self.evaluate_condition(condition) {
                    self.execute_tasks(workflow_id, if_branch).await
                } else if let Some(branch) = else_branch {
                    self.execute_tasks(workflow_id, branch).await
                } else {
                    Ok(Vec::new())
                }
            }
            ControlFlowNode::Loop {
                max_iterations,
                tasks,
            } => {
                let mut executed = Vec::new();
                for _ in 0..*max_iterations {
                    if self.context.get("continue") == Some(&Value::Bool(false)) {
                        break;
                    }
                    executed.extend(self.execute_tasks(workflow_id, tasks).await?);
                }
                Ok(executed)
            }
        }
    }

    async fn execute_tasks(
        &mut self,
        workflow_id: &str,
        tasks: &[TaskDef],
    ) -> Result<Vec<String>, String> {
        let mut executed = Vec::new();
        for task in tasks {
            self.execute_task(workflow_id, task).await?;
            executed.push(task.id.clone());
        }
        Ok(executed)
    }

    async fn execute_parallel(
        &mut self,
        workflow_id: &str,
        branches: &[Vec<TaskDef>],
    ) -> Result<Vec<String>, String> {
        let mut handles = Vec::new();
        for branch in branches {
            for task in branch {
                self.ensure_approval(workflow_id, task)?;
                handles.push((task.id.clone(), self.task_pool.execute(task.clone())));
            }
        }

        let mut executed = Vec::new();
        for (task_id, handle) in handles {
            handle.await.map_err(|e| e.to_string())??;
            executed.push(task_id);
        }
        Ok(executed)
    }

    async fn execute_task(&mut self, workflow_id: &str, task: &TaskDef) -> Result<(), String> {
        self.ensure_approval(workflow_id, task)?;
        self.task_pool
            .execute(task.clone())
            .await
            .map_err(|e| e.to_string())?
    }

    fn ensure_approval(&mut self, workflow_id: &str, task: &TaskDef) -> Result<(), String> {
        if !task.approval_required {
            return Ok(());
        }
        if self.approvals.iter().any(|approval| {
            approval.workflow_id == workflow_id
                && approval.step_id == task.id
                && approval.status == ApprovalStatus::Approved
        }) {
            return Ok(());
        }

        if !self
            .approvals
            .iter()
            .any(|approval| approval.workflow_id == workflow_id && approval.step_id == task.id)
        {
            self.approvals.push(WorkflowApproval {
                workflow_id: workflow_id.to_string(),
                step_id: task.id.clone(),
                action: Self::action_name(&task.action),
                status: ApprovalStatus::Pending,
                assigned_to: None,
                comment: None,
            });
        }
        Err(format!("Workflow step '{}' is pending approval", task.id))
    }

    fn evaluate_condition(&self, condition: &str) -> bool {
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return condition.trim().eq_ignore_ascii_case("true");
        }

        let key = parts[0].trim_start_matches("$context.");
        let Some(value) = self.context.get(key) else {
            return false;
        };
        Self::compare(value, parts[1], parts[2])
    }

    fn compare(value: &Value, op: &str, expected: &str) -> bool {
        if let Some(left) = value.as_f64() {
            let Ok(right) = expected.parse::<f64>() else {
                return false;
            };
            return match op {
                ">" => left > right,
                ">=" => left >= right,
                "<" => left < right,
                "<=" => left <= right,
                "==" => (left - right).abs() < f64::EPSILON,
                "!=" => (left - right).abs() >= f64::EPSILON,
                _ => false,
            };
        }

        let left = value.as_str().unwrap_or_default();
        match op {
            "==" => left == expected.trim_matches('"'),
            "!=" => left != expected.trim_matches('"'),
            _ => false,
        }
    }

    fn action_name(action: &WorkflowAction) -> String {
        match action {
            WorkflowAction::LLMCall { .. } => "llm_call",
            WorkflowAction::ToolCall { .. } => "tool_call",
            WorkflowAction::AgentCall { .. } => "agent_call",
            WorkflowAction::TeamCall { .. } => "team_call",
            WorkflowAction::SubWorkflow { .. } => "sub_workflow",
            WorkflowAction::CodeExec { .. } => "code_exec",
            WorkflowAction::KnowledgeQuery { .. } => "knowledge_query",
            WorkflowAction::HumanApproval { .. } => "human_approval",
            WorkflowAction::HumanInput { .. } => "human_input",
            WorkflowAction::Notification { .. } => "notification",
            WorkflowAction::Condition { .. } => "condition",
            WorkflowAction::Loop { .. } => "loop",
            WorkflowAction::Wait { .. } => "wait",
            WorkflowAction::Fork { .. } => "fork",
            WorkflowAction::Join => "join",
        }
        .to_string()
    }
}
