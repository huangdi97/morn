//! Orchestrator integration tests.
use crate::core::error::MornError;
use super::*;

fn create_test_team(id: &str, mode: CollaborationMode) -> TeamDef {
    TeamDef {
        id: id.to_string(),
        name: format!("Team {}", id),
        members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
        mode,
        consensus: ConsensusMechanism::CeoDecides,
    }
}

#[test]
fn test_create_team() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = create_test_team("team-1", CollaborationMode::Chain);
    let id = orch.create_team(team).unwrap();
    assert_eq!(id, "team-1");
}

#[test]
fn test_duplicate_team() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = create_test_team("team-1", CollaborationMode::Chain);
    orch.create_team(team).unwrap();
    let dup = create_test_team("team-1", CollaborationMode::Broadcast);
    assert!(orch.create_team(dup).is_err());
}

#[test]
fn test_chain_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = create_test_team("team-chain", CollaborationMode::Chain);
    orch.create_team(team).unwrap();
    let result = orch.run_team("team-chain", "hello").unwrap();
    assert_eq!(result.mode, "chain");
    assert_eq!(result.outputs.len(), 3);
}

#[test]
fn test_broadcast_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = create_test_team("team-bc", CollaborationMode::Broadcast);
    orch.create_team(team).unwrap();
    let result = orch.run_team("team-bc", "hello").unwrap();
    assert_eq!(result.outputs.len(), 3);
}

#[test]
fn test_consensus_vote() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = TeamDef {
        id: "team-vote".into(),
        name: "Vote Team".into(),
        members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
        mode: CollaborationMode::Voting,
        consensus: ConsensusMechanism::Vote,
    };
    orch.create_team(team).unwrap();
    let result = orch.run_team("team-vote", "evaluate").unwrap();
    assert_eq!(result.outputs.len(), 3);
    assert!(!result.consensus_output.is_empty());
}

#[test]
fn test_list_teams() {
    let mut orch = Orchestrator::new(None, None, None);
    orch.create_team(create_test_team("t1", CollaborationMode::Chain))
        .unwrap();
    orch.create_team(create_test_team("t2", CollaborationMode::Broadcast))
        .unwrap();
    assert_eq!(orch.list_teams().len(), 2);
}

#[test]
fn test_delete_team() {
    let mut orch = Orchestrator::new(None, None, None);
    orch.create_team(create_test_team("t1", CollaborationMode::Chain))
        .unwrap();
    orch.delete_team("t1").unwrap();
    assert!(orch.get_team("t1").is_none());
}

#[test]
fn test_routing_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = create_test_team("team-route", CollaborationMode::Routing);
    orch.create_team(team).unwrap();
    let result = orch.run_team("team-route", "route me").unwrap();
    assert_eq!(result.outputs.len(), 1);
}

#[test]
fn test_consensus_mechanisms() {
    let mut orch = Orchestrator::new(None, None, None);
    for (mode_name, mechanism) in [
        ("ceo", ConsensusMechanism::CeoDecides),
        ("vote", ConsensusMechanism::Vote),
        ("veto", ConsensusMechanism::MungerVeto),
        ("synth", ConsensusMechanism::AutoSynthesis),
    ] {
        let team = TeamDef {
            id: format!("team-{}", mode_name),
            name: format!("{} Team", mode_name),
            members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
            mode: CollaborationMode::Voting,
            consensus: mechanism,
        };
        orch.create_team(team).unwrap();
        let result = orch
            .run_team(&format!("team-{}", mode_name), "test")
            .unwrap();
        assert!(!result.consensus_output.is_empty());
    }
}

#[test]
fn test_register_expert() {
    let mut orch = Orchestrator::new(None, None, None);
    let expert = ExpertSpec {
        id: "expert-data".into(),
        name: "Data Analyst".into(),
        domain: "data".into(),
        persona_id: "preset-analyst".into(),
        description: "Analyzes data and finds patterns".into(),
    };
    let id = orch.register_expert(expert).unwrap();
    assert_eq!(id, "expert-data");
    assert_eq!(orch.list_experts().len(), 1);
}

#[test]
fn test_find_experts_for_task() {
    let mut orch = Orchestrator::new(None, None, None);
    orch.register_expert(ExpertSpec {
        id: "expert-data".into(),
        name: "Data Analyst".into(),
        domain: "data".into(),
        persona_id: "preset-analyst".into(),
        description: "Analyzes data".into(),
    })
    .unwrap();
    orch.register_expert(ExpertSpec {
        id: "expert-code".into(),
        name: "Coder".into(),
        domain: "code".into(),
        persona_id: "preset-coder".into(),
        description: "Writes code".into(),
    })
    .unwrap();
    orch.register_expert(ExpertSpec {
        id: "expert-write".into(),
        name: "Writer".into(),
        domain: "writing".into(),
        persona_id: "preset-writer".into(),
        description: "Creates content".into(),
    })
    .unwrap();

    let found = orch.find_experts_for_task("need data analysis and code help", 2);
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|e| e.id == "expert-data"));
}

#[test]
fn test_dispatch_agent() {
    let orch = Orchestrator::new(None, None, None);
    let result = orch.dispatch_agent("test-agent", "hello world").unwrap();
    assert_eq!(result.agent_id, "test-agent");
    assert!(result.confidence > 0.0);
}

#[test]
fn test_run_manager_expert_no_experts() {
    let orch = Orchestrator::new(None, None, None);
    let result = orch.run_manager_expert("manager-1", "unknown task");
    assert!(result.is_err());
}

#[test]
fn test_unregister_expert() {
    let mut orch = Orchestrator::new(None, None, None);
    orch.register_expert(ExpertSpec {
        id: "expert-1".into(),
        name: "E1".into(),
        domain: "test".into(),
        persona_id: "preset-assistant".into(),
        description: "test".into(),
    })
    .unwrap();
    assert!(orch.unregister_expert("expert-1").is_ok());
    assert!(orch.unregister_expert("nonexistent").is_err());
}
