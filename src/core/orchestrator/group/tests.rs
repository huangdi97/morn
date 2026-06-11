//! Agent group tests.
use super::*;

#[test]
fn test_agent_group_creation() {
    let group = AgentGroup::new(
        "group-1",
        vec!["agent-a".to_string(), "agent-b".to_string()],
        CollaborationMode::Chain,
        "ws-1",
    );
    assert_eq!(group.group_id, "group-1");
    assert_eq!(group.agent_ids.len(), 2);
}

#[test]
fn test_agent_group_with_config() {
    let config = GroupConfig {
        max_concurrency: 2,
        timeout_secs: 600,
        ..GroupConfig::default()
    };
    let group = AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Broadcast,
        "ws1",
    )
    .with_config(config);
    assert_eq!(group.config.max_concurrency, 2);
    assert_eq!(group.config.timeout_secs, 600);
}

#[test]
fn test_group_executor_register_and_get() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Chain,
        "ws1",
    ));
    let group = executor.get_group("g1");
    assert!(group.is_some());
    assert_eq!(group.unwrap().group_id, "g1");
}

#[test]
fn test_group_executor_get_nonexistent() {
    let executor = GroupExecutor::new();
    assert!(executor.get_group("nonexistent").is_none());
}

#[test]
fn test_group_executor_status() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string(), "a2".to_string()],
        CollaborationMode::Voting,
        "ws1",
    ));
    let status = executor.group_status("g1").unwrap();
    assert!(status.contains("g1"));
    assert!(status.contains("Voting"));
}

#[test]
fn test_workspace_creation() {
    let mut ws = Workspace::new("ws-1", "Test Workspace");
    ws.add_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Chain,
        "ws-1",
    ));
    ws.add_cron_task(CronTask {
        id: "cron-1".to_string(),
        expression: "0 0 * * *".to_string(),
        group_id: "g1".to_string(),
        enabled: true,
    });
    assert_eq!(ws.groups.len(), 1);
    assert_eq!(ws.cron_tasks.len(), 1);
}

#[test]
fn test_project_lifecycle() {
    let mut project = Project::new("proj-1", "Test Project", "Build a research system");
    assert_eq!(project.lifecycle_state, LifecycleState::Draft);

    project.activate();
    assert_eq!(project.lifecycle_state, LifecycleState::Active);

    project.pause();
    assert_eq!(project.lifecycle_state, LifecycleState::Paused);

    project.complete();
    assert_eq!(project.lifecycle_state, LifecycleState::Completed);
}

#[test]
fn test_project_with_workspace() {
    let mut project = Project::new("p1", "Research", "Research agents");
    let ws = Workspace::new("ws1", "Data Collection");
    project.add_workspace(ws);
    assert_eq!(project.workspaces.len(), 1);
}

#[test]
fn test_group_executor_execute_chain() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec![
            "agent-a".to_string(),
            "agent-b".to_string(),
            "agent-c".to_string(),
        ],
        CollaborationMode::Chain,
        "ws1",
    ));
    let result = executor.execute_group("g1", "test input");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}

#[test]
fn test_group_metrics_after_execution() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Chain,
        "ws1",
    ));
    executor.execute_group("g1", "input").unwrap();
    let metrics = executor.group_metrics("g1").unwrap();
    assert_eq!(metrics.execution_count, 1);
    assert!(metrics.total_cost_estimate > 0.0);
    assert!(metrics.last_executed_at.is_some());
}

#[test]
fn test_group_cost_estimate() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string(), "a2".to_string()],
        CollaborationMode::Voting,
        "ws1",
    ));
    let cost = executor.group_cost_estimate("g1").unwrap();
    assert!((cost - 0.16).abs() < 1e-6);
}

#[test]
fn test_group_summary() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g-summary",
        vec!["agent-x".to_string()],
        CollaborationMode::Broadcast,
        "ws-main",
    ));
    let summary = executor.group_summary("g-summary").unwrap();
    assert!(summary.contains("g-summary"));
    assert!(summary.contains("Broadcast"));
    assert!(summary.contains("¥"));
}

#[test]
fn test_group_summary_nonexistent() {
    let executor = GroupExecutor::new();
    assert!(executor.group_summary("no-such-group").is_err());
}

#[test]
fn test_all_metrics() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Chain,
        "ws1",
    ));
    executor.register_group(AgentGroup::new(
        "g2",
        vec!["a2".to_string()],
        CollaborationMode::Voting,
        "ws1",
    ));
    assert_eq!(executor.all_metrics().len(), 2);
}

#[test]
fn test_execution_tracks_metrics_correctly() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g-multi",
        vec!["a1".to_string(), "a2".to_string()],
        CollaborationMode::Blackboard,
        "ws1",
    ));
    executor.execute_group("g-multi", "run1").unwrap();
    executor.execute_group("g-multi", "run2").unwrap();
    let metrics = executor.group_metrics("g-multi").unwrap();
    assert_eq!(metrics.execution_count, 2);
}

#[test]
fn test_get_metrics_mut() {
    let mut executor = GroupExecutor::new();
    executor.register_group(AgentGroup::new(
        "g1",
        vec!["a1".to_string()],
        CollaborationMode::Chain,
        "ws1",
    ));
    {
        let m = executor.get_metrics_mut("g1").unwrap();
        m.execution_count = 5;
    }
    assert_eq!(executor.group_metrics("g1").unwrap().execution_count, 5);
}
