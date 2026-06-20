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
    assert!(
        all.len() >= 8,
        "Expected at least 8 templates, got {}",
        all.len()
    );
}

#[test]
fn test_find_template_coder() {
    let tpl = find_template("程序员").unwrap();
    assert_eq!(tpl.persona, "coder");
}

#[test]
fn test_persona_preset() {
    let p = morn::component::persona::presets::get_preset_persona("preset-researcher");
    assert!(p.is_some() && !p.unwrap().name.is_empty());
}

#[test]
fn test_coo_decision() {
    use morn::core::supervisor::{DecisionLevel, Supervisor};
    let sup = Supervisor::new(None, None);
    assert_eq!(sup.decide_level("hello"), DecisionLevel::L1DirectAnswer);
}

#[test]
fn test_storage_agent_crud() {
    use morn::core::storage::{AgentRecord, Storage};
    let storage = Storage::new_in_memory().unwrap();
    let agent = AgentRecord {
        id: "test-agent".into(),
        name: "Test".into(),
        component_type: "agent".into(),
        config_json: None,
        status: "active".into(),
        trust_score: 1.0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: None,
        current_version: "1.0".into(),
        update_available: false,
    };
    storage.insert_agent(&agent).unwrap();
    let got = storage.get_agent("test-agent").unwrap().unwrap();
    assert_eq!(got.name, "Test");
}

#[test]
fn test_workflow_templates_count() {
    use morn::core::workflow::WorkflowTemplate;
    let count = WorkflowTemplate::list_builtin().len();
    assert!(count >= 13);
}

#[test]
fn test_registry_create_skeleton() {
    use morn::core::registry::Registry;
    let mut reg = Registry::new(None, None);
    let cap = reg.create_skeleton("test-comp", "tool").unwrap();
    assert!(cap.id.contains("test-comp"));
}

// ── Agent creation + retrieval ─────────────────────────────────────────────

#[test]
fn test_registry_get_template() {
    use morn::core::registry::Registry;
    let reg = Registry::new(None, None);
    let tpl = reg.get_template("general-assistant");
    assert!(tpl.is_some());
    let tpl = tpl.unwrap();
    assert!(
        !tpl.version.is_empty(),
        "Template 'general-assistant' should have a non-empty version, got '{}'",
        tpl.version
    );
}

#[test]
fn test_registry_list_templates() {
    use morn::core::registry::Registry;
    let reg = Registry::new(None, None);
    let templates = reg.list_templates();
    assert!(
        templates.len() >= 6,
        "Expected at least 6 templates, got {}",
        templates.len()
    );
}

#[test]
fn test_registry_register_dynamic_and_get() {
    use morn::core::registry::{Capability, Registry};
    let mut reg = Registry::new(None, None);
    let cap = Capability {
        id: "test-cap-dynamic".into(),
        version: "1.0.0".into(),
        name: "Dynamic Cap".into(),
        domain: "test".into(),
        actions: vec!["ping".into()],
        description: "test capability".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    };
    reg.register_dynamic(cap).unwrap();
    let got = reg.get("test-cap-dynamic");
    assert!(got.is_some());
    assert_eq!(got.unwrap().name, "Dynamic Cap");
}

#[test]
fn test_registry_register_dynamic_duplicate_fails() {
    use morn::core::registry::{Capability, Registry};
    let mut reg = Registry::new(None, None);
    let cap = Capability {
        id: "dup-cap".into(),
        version: "0.1.0".into(),
        name: "Original".into(),
        domain: "test".into(),
        actions: vec!["act".into()],
        description: "original".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    };
    reg.register_dynamic(cap).unwrap();
    let dup = Capability {
        id: "dup-cap".into(),
        version: "0.2.0".into(),
        name: "Duplicate".into(),
        domain: "test".into(),
        actions: vec!["act".into()],
        description: "duplicate".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    };
    assert!(reg.register_dynamic(dup).is_err());
}

#[test]
fn test_registry_unregister() {
    use morn::core::registry::{Capability, Registry};
    let mut reg = Registry::new(None, None);
    reg.register_dynamic(Capability {
        id: "rm-cap".into(),
        version: "0.1.0".into(),
        name: "To Remove".into(),
        domain: "test".into(),
        actions: vec![],
        description: "".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    })
    .unwrap();
    let removed = reg.unregister("rm-cap");
    assert!(removed.is_some());
    assert!(reg.get("rm-cap").is_none());
}

#[test]
fn test_registry_list_all_and_find_by_domain() {
    use morn::core::registry::{Capability, Registry};
    let mut reg = Registry::new(None, None);
    reg.register_dynamic(Capability {
        id: "list-cap-1".into(),
        version: "0.1.0".into(),
        name: "List1".into(),
        domain: "domain-a".into(),
        actions: vec!["act1".into()],
        description: "".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    })
    .unwrap();
    reg.register_dynamic(Capability {
        id: "list-cap-2".into(),
        version: "0.1.0".into(),
        name: "List2".into(),
        domain: "domain-b".into(),
        actions: vec!["act2".into()],
        description: "".into(),
        trust_score: 50.0,
        total_calls: 0,
        success_calls: 0,
        avg_latency_ms: 0.0,
        visibility: "public".into(),
        owner_id: None,
        team_id: None,
        daily_quota: 0,
    })
    .unwrap();
    assert!(reg.list_all().len() >= 3);
    assert_eq!(reg.find_by_domain("domain-a").len(), 1);
    assert_eq!(reg.find_by_action("act2").len(), 1);
}

// ── Team management flow ───────────────────────────────────────────────────

#[test]
fn test_team_full_lifecycle() {
    use morn::core::orchestrator::{CollaborationMode, ConsensusMechanism, Orchestrator, TeamDef};
    let mut orch = Orchestrator::new(None, None, None);

    let team = TeamDef {
        id: "lifecycle-team".into(),
        name: "Lifecycle".into(),
        members: vec!["a".into(), "b".into()],
        mode: CollaborationMode::Broadcast,
        consensus: ConsensusMechanism::CeoDecides,
    };
    orch.create_team(team).unwrap();
    assert_eq!(orch.list_teams().len(), 1);
    assert!(orch.get_team("lifecycle-team").is_some());

    orch.delete_team("lifecycle-team").unwrap();
    assert!(orch.get_team("lifecycle-team").is_none());
    assert_eq!(orch.list_teams().len(), 0);
}

#[test]
fn test_team_duplicate_rejected() {
    use morn::core::orchestrator::{CollaborationMode, ConsensusMechanism, Orchestrator, TeamDef};
    let mut orch = Orchestrator::new(None, None, None);
    let team = TeamDef {
        id: "dup-team".into(),
        name: "Dup".into(),
        members: vec!["x".into()],
        mode: CollaborationMode::Chain,
        consensus: ConsensusMechanism::CeoDecides,
    };
    orch.create_team(team).unwrap();
    let dup = TeamDef {
        id: "dup-team".into(),
        name: "Dup".into(),
        members: vec!["y".into()],
        mode: CollaborationMode::Voting,
        consensus: ConsensusMechanism::Vote,
    };
    assert!(orch.create_team(dup).is_err());
}

#[test]
fn test_team_delete_nonexistent_returns_error() {
    use morn::core::orchestrator::Orchestrator;
    let mut orch = Orchestrator::new(None, None, None);
    let result = orch.delete_team("does-not-exist");
    assert!(result.is_err());
}

// ── Message sending pipeline (Supervisor) ──────────────────────────────────

#[test]
fn test_supervisor_decision_levels() {
    use morn::core::supervisor::{DecisionLevel, Supervisor};
    let sup = Supervisor::new(None, None);
    assert_eq!(sup.decide_level("hello"), DecisionLevel::L1DirectAnswer);
    assert_eq!(
        sup.decide_level("search for AI news"),
        DecisionLevel::L2SingleTool
    );
    assert_eq!(
        sup.decide_level("calculate 2+2"),
        DecisionLevel::L2SingleTool
    );
    // 以下断言依赖关键词匹配启发式，修改匹配规则时可能需要同步更新
    assert_eq!(
        sup.decide_level("create a report"),
        DecisionLevel::L5Workflow
    );
    assert_eq!(sup.decide_level("analysis"), DecisionLevel::L5Workflow);
}

#[test]
fn test_supervisor_record_and_history() {
    use morn::core::supervisor::Supervisor;
    let mut sup = Supervisor::new(None, None);
    sup.record_turn("user", "hello");
    sup.record_turn("assistant", "hi there");
    assert_eq!(sup.history().len(), 2);
    assert_eq!(sup.turn_count(), 0);
}

#[test]
fn test_supervisor_build_context() {
    use morn::core::supervisor::Supervisor;
    let mut sup = Supervisor::new(None, None);
    sup.record_turn("user", "what is the weather?");
    let ctx = sup.build_context("tell me now");
    assert!(ctx.contains("what is the weather?"));
    assert!(ctx.contains("tell me now"));
}

#[test]
fn test_supervisor_clear_history() {
    use morn::core::supervisor::Supervisor;
    let mut sup = Supervisor::new(None, None);
    sup.record_turn("user", "hello");
    sup.clear_history();
    assert!(sup.history().is_empty());
    assert_eq!(sup.turn_count(), 0);
}

#[test]
fn test_supervisor_build_team_from_nl() {
    use morn::core::supervisor::Supervisor;
    let sup = Supervisor::new(None, None);
    let team = sup.build_team_from_nl("code review team").unwrap();
    assert!(!team.members.is_empty());
    assert!(
        !team.members.is_empty(),
        "Expected team built from 'code review team' to have members"
    );
    assert!(
        team.id.contains("code-review") || team.id.contains("review"),
        "team.id should mention code-review or review, got '{}'",
        team.id
    );
}

// ── Hub publish basic flow ─────────────────────────────────────────────────

#[test]
fn test_marketplace_publish_and_list() {
    use morn::core::storage::Storage;
    use morn::market::{Listing, Marketplace};
    let storage = Storage::new_in_memory().unwrap();
    let market = Marketplace::new(storage.clone());
    let listing = Listing {
        id: "test-listing-1".into(),
        item_type: "tool".into(),
        name: "Test Tool".into(),
        description: "A test listing".into(),
        price: Some(0.0),
        price_model: "free".into(),
        author: "Test Author".into(),
        rating: 0.0,
        downloads: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".into(),
        screenshots: "".into(),
        category: "general".into(),
        requires: vec![],
        verified: false,
        updated_at: "".into(),
    };
    market.publish(listing).unwrap();
    let listings = market.list(None);
    assert!(listings.iter().any(|l| l.id == "test-listing-1"));
}

#[test]
fn test_marketplace_get_listing_by_id() {
    use morn::core::storage::Storage;
    use morn::market::{Listing, Marketplace};
    let storage = Storage::new_in_memory().unwrap();
    let market = Marketplace::new(storage.clone());
    let listing = Listing {
        id: "get-test-listing".into(),
        item_type: "knowledge".into(),
        name: "Get Test".into(),
        description: "Test get by id".into(),
        price: Some(1.0),
        price_model: "free".into(),
        author: "Author".into(),
        rating: 4.0,
        downloads: 10,
        created_at: chrono::Utc::now().to_rfc3339(),
        version: "2.0.0".into(),
        screenshots: "".into(),
        category: "data".into(),
        requires: vec![],
        verified: false,
        updated_at: "".into(),
    };
    market.publish(listing).unwrap();
    let got = market.get("get-test-listing");
    assert!(got.is_some());
    assert_eq!(got.unwrap().name, "Get Test");
}

#[test]
fn test_marketplace_listing_not_found() {
    use morn::core::storage::Storage;
    use morn::market::Marketplace;
    let storage = Storage::new_in_memory().unwrap();
    let market = Marketplace::new(storage);
    assert!(market.get("nonexistent").is_none());
}

#[test]
fn test_marketplace_has_builtin_listings() {
    use morn::core::storage::Storage;
    use morn::market::Marketplace;
    let storage = Storage::new_in_memory().unwrap();
    let market = Marketplace::new(storage);
    let all = market.list(None);
    assert!(!all.is_empty());
    assert!(all.iter().any(|l| l.author == "Morn Labs"));
}

// ── Config read/write ──────────────────────────────────────────────────────

#[test]
fn test_config_set_and_get() {
    use morn::core::storage::Storage;
    let storage = Storage::new_in_memory().unwrap();
    storage.set_setting("theme", "dark").unwrap();
    assert_eq!(
        storage.get_setting("theme").unwrap().as_deref(),
        Some("dark")
    );
}

#[test]
fn test_config_update_existing() {
    use morn::core::storage::Storage;
    let storage = Storage::new_in_memory().unwrap();
    storage.set_setting("lang", "en").unwrap();
    storage.set_setting("lang", "zh").unwrap();
    assert_eq!(storage.get_setting("lang").unwrap().as_deref(), Some("zh"));
}

#[test]
fn test_config_get_nonexistent_returns_none() {
    use morn::core::storage::Storage;
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.get_setting("no_such_key").unwrap().is_none());
}

#[test]
fn test_config_multiple_settings() {
    use morn::core::storage::Storage;
    let storage = Storage::new_in_memory().unwrap();
    storage.set_setting("key1", "val1").unwrap();
    storage.set_setting("key2", "val2").unwrap();
    storage.set_setting("key3", "val3").unwrap();
    assert_eq!(
        storage.get_setting("key1").unwrap().as_deref(),
        Some("val1")
    );
    assert_eq!(
        storage.get_setting("key2").unwrap().as_deref(),
        Some("val2")
    );
    assert_eq!(
        storage.get_setting("key3").unwrap().as_deref(),
        Some("val3")
    );
}
