//! Workflow engine tests.
use super::{ControlFlowNode, WorkflowAction, WorkflowEngine, WorkflowStep};
use crate::core::thread_pool::TaskPool;
use std::collections::HashMap;

fn step(id: &str, approval_required: bool) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        action: WorkflowAction::Notification {
            channel: "test".to_string(),
            message: id.to_string(),
        },
        depends_on: vec![],
        timeout_secs: 1,
        retry_count: 0,
        approval_required,
        input_mapping: HashMap::new(),
        output_mapping: HashMap::new(),
    }
}

#[tokio::test]
async fn executes_sequential_tasks() {
    let mut engine = WorkflowEngine::new(TaskPool::default());
    let executed = engine
        .execute_node("wf-1", &ControlFlowNode::Sequential(vec![step("a", false)]))
        .await
        .unwrap();

    assert_eq!(executed, vec!["a"]);
}

#[tokio::test]
async fn executes_parallel_branches() {
    let mut engine = WorkflowEngine::new(TaskPool::default());
    let executed = engine
        .execute_node(
            "wf-1",
            &ControlFlowNode::Parallel(vec![vec![step("a", false)], vec![step("b", false)]]),
        )
        .await
        .unwrap();

    assert_eq!(executed.len(), 2);
    assert!(executed.contains(&"a".to_string()));
    assert!(executed.contains(&"b".to_string()));
}

#[tokio::test]
async fn conditional_selects_matching_branch() {
    let mut engine = WorkflowEngine::new(TaskPool::default());
    engine
        .context_mut()
        .insert("value".into(), serde_json::json!(6));

    let executed = engine
        .execute_node(
            "wf-1",
            &ControlFlowNode::Conditional {
                condition: "$context.value > 5".to_string(),
                if_branch: vec![step("if", false)],
                else_branch: Some(vec![step("else", false)]),
            },
        )
        .await
        .unwrap();

    assert_eq!(executed, vec!["if"]);
}

#[tokio::test]
async fn loop_repeats_tasks() {
    let mut engine = WorkflowEngine::new(TaskPool::default());
    let executed = engine
        .execute_node(
            "wf-1",
            &ControlFlowNode::Loop {
                max_iterations: 3,
                tasks: vec![step("tick", false)],
            },
        )
        .await
        .unwrap();

    assert_eq!(executed, vec!["tick", "tick", "tick"]);
}

#[tokio::test]
async fn approval_required_step_pauses_execution() {
    let mut engine = WorkflowEngine::new(TaskPool::default());
    let err = engine
        .execute_node(
            "wf-1",
            &ControlFlowNode::Sequential(vec![step("gate", true)]),
        )
        .await
        .unwrap_err();

    assert!(err.contains("pending approval"));
    assert_eq!(engine.pending_approvals().len(), 1);

    engine
        .approve_step("wf-1", "gate", true, Some("ok".to_string()))
        .unwrap();
    let executed = engine
        .execute_node(
            "wf-1",
            &ControlFlowNode::Sequential(vec![step("gate", true)]),
        )
        .await
        .unwrap();
    assert_eq!(executed, vec!["gate"]);
}
