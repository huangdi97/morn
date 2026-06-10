use super::*;

#[test]
fn test_supervisor_build_context() {
    let supervisor = Supervisor::new(None, None);
    let context = supervisor.build_context("hello");
    assert!(context.contains("You are Morn"));
    assert!(context.contains("[Current]"));
    assert!(context.contains("hello"));
}

#[test]
fn test_supervisor_record_and_context() {
    let mut supervisor = Supervisor::new(None, None);
    supervisor.record_turn("user", "hi");
    supervisor.record_turn("assistant", "hello!");
    let context = supervisor.build_context("how are you?");
    assert!(context.contains("hi"));
    assert!(context.contains("hello!"));
    assert!(context.contains("how are you?"));
}

#[test]
fn test_decide_level_simple() {
    let supervisor = Supervisor::new(None, None);
    assert_eq!(
        supervisor.decide_level("hello"),
        DecisionLevel::L1DirectAnswer
    );
    assert_eq!(
        supervisor.decide_level("thanks"),
        DecisionLevel::L1DirectAnswer
    );
}

#[test]
fn test_decide_level_tool() {
    let supervisor = Supervisor::new(None, None);
    assert_eq!(
        supervisor.decide_level("search for AI news"),
        DecisionLevel::L2SingleTool
    );
    assert_eq!(
        supervisor.decide_level("calculate 2+2"),
        DecisionLevel::L2SingleTool
    );
}

#[test]
fn test_decide_level_workflow() {
    let supervisor = Supervisor::new(None, None);
    assert_eq!(
        supervisor.decide_level("create a report"),
        DecisionLevel::L5Workflow
    );
    assert_eq!(
        supervisor.decide_level("analysis"),
        DecisionLevel::L5Workflow
    );
}

#[test]
fn test_decide_level_studio() {
    let supervisor = Supervisor::new(None, None);
    assert_eq!(
        supervisor.decide_level("create an agent"),
        DecisionLevel::L6JumpToStudio
    );
}

#[test]
fn test_decide_level_default() {
    let supervisor = Supervisor::new(None, None);
    assert_eq!(
        supervisor.decide_level("tell me about quantum physics"),
        DecisionLevel::L3SingleAgent
    );
}

#[test]
fn test_coo_mode() {
    let mut supervisor = Supervisor::new(None, None);
    assert_eq!(*supervisor.mode(), Mode::Proactive);
    supervisor.set_mode(Mode::Safe);
    assert_eq!(*supervisor.mode(), Mode::Safe);
}

#[test]
fn test_decision_override_is_recorded() {
    let mut supervisor = Supervisor::new(None, None);
    supervisor.override_decision(DecisionLevel::L4Team, OverrideScope::Session);

    assert_eq!(
        supervisor.decision_override().map(|o| &o.level),
        Some(&DecisionLevel::L4Team)
    );
}

#[test]
fn test_live_suggestion_uses_recent_user_turn_in_proactive_mode() {
    let mut supervisor = Supervisor::new(None, None);
    supervisor.record_turn("user", "please search docs");

    assert_eq!(
        supervisor.live_suggestion(),
        Some(DecisionLevel::L2SingleTool)
    );
}

#[test]
fn test_decide_reasoning() {
    let supervisor = Supervisor::new(None, None);
    let (level, _reasoning) = supervisor.decide("complex multi-step task");
    assert_eq!(level, DecisionLevel::L4Team);
}

#[test]
fn test_create_team_from_nl_single_agent() {
    let supervisor = Supervisor::new(None, None);
    let chat_fn = |_prompt: &str, _system: &str| Ok("SINGLE".to_string());
    let result = supervisor.create_team_from_nl("simple greeting", &chat_fn);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Single agent"));
}

#[test]
fn test_create_team_from_nl_preset_research() {
    let supervisor = Supervisor::new(None, None);
    let chat_fn = |_prompt: &str, _system: &str| Ok("TEAM".to_string());
    let result = supervisor.create_team_from_nl("need research and analysis", &chat_fn);
    assert!(result.is_ok());
    let team = result.unwrap();
    assert_eq!(team.name, "Research Team");
}

#[test]
fn test_create_team_from_nl_preset_code() {
    let supervisor = Supervisor::new(None, None);
    let chat_fn = |_prompt: &str, _system: &str| Ok("TEAM".to_string());
    let result = supervisor.create_team_from_nl("build a web app", &chat_fn);
    assert!(result.is_ok());
    let team = result.unwrap();
    assert_eq!(team.name, "Development Team");
}

#[test]
fn test_create_team_from_nl_llm_generated() {
    let supervisor = Supervisor::new(None, None);
    let json_response = r#"{"id":"team-custom","name":"Custom Team","members":["agent-a","agent-b"],"mode":"Chain","consensus":"CeoDecides"}"#;
    let chat_fn = move |prompt: &str, _system: &str| {
        if prompt.contains("SINGLE") || prompt.contains("TEAM") {
            Ok("TEAM".to_string())
        } else {
            Ok(json_response.to_string())
        }
    };
    let result = supervisor.create_team_from_nl("something totally unique and custom", &chat_fn);
    assert!(result.is_ok());
    let team = result.unwrap();
    assert_eq!(team.id, "team-custom");
    assert_eq!(team.members.len(), 2);
}

#[test]
fn test_build_team_from_nl_uses_local_builder() {
    let supervisor = Supervisor::new(None, None);
    let team = supervisor
        .build_team_from_nl("devops deploy monitor")
        .unwrap();

    assert_eq!(team.id, "preset-devops");
    assert_eq!(team.members.len(), 3);
}

#[test]
fn test_modify_rule_from_nl_add() {
    let storage = Storage::new_in_memory().unwrap();
    let supervisor = Supervisor::new(Some(storage), None);
    let result =
        supervisor.modify_rule_from_nl("add | deploy | L4 | contains 'deploy' | require_approval");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Rule added");
}

#[test]
fn test_modify_rule_from_nl_list() {
    let storage = Storage::new_in_memory().unwrap();
    let supervisor = Supervisor::new(Some(storage), None);
    supervisor
        .modify_rule_from_nl("add | search | L2 | contains 'search' | auto_execute")
        .unwrap();
    let result = supervisor.modify_rule_from_nl("list all");
    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains("search"));
}

#[test]
fn test_modify_rule_from_nl_find() {
    let storage = Storage::new_in_memory().unwrap();
    let supervisor = Supervisor::new(Some(storage), None);
    supervisor
        .modify_rule_from_nl("add | search | L2 | contains 'search' | auto_execute")
        .unwrap();
    let result = supervisor.modify_rule_from_nl("find search");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("L2"));
}

#[test]
fn test_modify_rule_from_nl_delete() {
    let storage = Storage::new_in_memory().unwrap();
    let supervisor = Supervisor::new(Some(storage), None);
    supervisor
        .modify_rule_from_nl("add | test | L1 | test | none")
        .unwrap();
    supervisor
        .modify_rule_from_nl("add | test2 | L2 | test2 | none")
        .unwrap();
    let rules_before = supervisor.modify_rule_from_nl("list all").unwrap();
    let rules: Vec<crate::core::decision_rules::DecisionRule> =
        serde_json::from_str(&rules_before).unwrap();
    assert_eq!(rules.len(), 2);
    supervisor
        .modify_rule_from_nl(&format!("delete {}", rules[0].id))
        .unwrap();
    let rules_after = supervisor.modify_rule_from_nl("list all").unwrap();
    let remaining: Vec<crate::core::decision_rules::DecisionRule> =
        serde_json::from_str(&rules_after).unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn test_modify_rule_from_nl_unknown() {
    let storage = Storage::new_in_memory().unwrap();
    let supervisor = Supervisor::new(Some(storage), None);
    let result = supervisor.modify_rule_from_nl("unknown command");
    assert!(result.is_err());
}

#[test]
fn test_modify_rule_from_nl_no_storage() {
    let supervisor = Supervisor::new(None, None);
    let result = supervisor.modify_rule_from_nl("list all");
    assert!(result.is_err());
}
