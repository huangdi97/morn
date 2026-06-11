use morn::core::agent_templates::{all_templates, find_template};
use morn::core::orchestrator::team_builder::nl_to_team;
use morn::core::orchestrator::{CollaborationMode, ConsensusMechanism, Orchestrator, TeamDef};

fn make_team(id: &str, mode: CollaborationMode) -> TeamDef {
    TeamDef {
        id: id.to_string(),
        name: format!("Team {}", id),
        members: vec!["alpha".into(), "beta".into(), "gamma".into()],
        mode,
        consensus: ConsensusMechanism::CeoDecides,
    }
}

#[test]
fn test_orchestrator_chain_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = make_team("team-chain", CollaborationMode::Chain);
    orch.create_team(team).unwrap();

    let result = orch.run_team("team-chain", "hello world").unwrap();

    assert_eq!(result.outputs.len(), 3);
    assert_eq!(result.mode, "chain");
}

#[test]
fn test_orchestrator_broadcast_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = make_team("team-bc", CollaborationMode::Broadcast);
    orch.create_team(team).unwrap();

    let result = orch.run_team("team-bc", "hello world").unwrap();

    assert_eq!(result.outputs.len(), 3);
    assert_eq!(result.mode, "broadcast");
}

#[test]
fn test_orchestrator_voting_mode() {
    let mut orch = Orchestrator::new(None, None, None);
    let team = TeamDef {
        id: "team-vote".into(),
        name: "Voting Team".into(),
        members: vec!["alpha".into(), "beta".into(), "gamma".into()],
        mode: CollaborationMode::Voting,
        consensus: ConsensusMechanism::Vote,
    };
    orch.create_team(team).unwrap();

    let result = orch.run_team("team-vote", "evaluate options").unwrap();

    assert_eq!(result.outputs.len(), 3);
    assert!(!result.consensus_output.is_empty());
    assert_eq!(result.mode, "voting");
}

#[test]
fn test_nl_to_team_stock_research() {
    let team = nl_to_team("stock analysis team").unwrap();
    assert!(!team.members.is_empty());
}

#[test]
fn test_nl_to_team_code_review() {
    let team = nl_to_team("code review team").unwrap();
    assert!(!team.members.is_empty());
}

#[test]
fn test_agent_templates_exist() {
    let all = all_templates();
    assert_eq!(all.len(), 8);
}

#[test]
fn test_find_template_coder() {
    let tpl = find_template("程序员").unwrap();
    assert_eq!(tpl.persona, "coder");
}

#[test]
fn test_persona_preset() {
    let p = morn::component::persona::presets_industry::preset_researcher();
    assert!(!p.name.is_empty());
}

#[test]
fn test_coo_decision() {
    use morn::core::supervisor::{DecisionLevel, Supervisor};
    let sup = Supervisor::new(None, None);
    assert_eq!(sup.decide_level("hello"), DecisionLevel::L1DirectAnswer);
}
