//! Storage core tests.
use super::*;
use crate::core::error::MornError;
use crate::market::{License, Listing, Transaction};

#[test]
fn test_storage_crud() {
    let storage = Storage::new_in_memory().unwrap();

    let agent = AgentRecord {
        id: "agent-1".to_string(),
        name: "Test Agent".to_string(),
        component_type: "agent".to_string(),
        config_json: Some("{}".to_string()),
        status: "active".to_string(),
        trust_score: 70.0,
        current_version: "1.0.0".into(),
        update_available: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: None,
    };
    storage.insert_agent(&agent).unwrap();
    let got = storage.get_agent("agent-1").unwrap().unwrap();
    assert_eq!(got.name, "Test Agent");
    assert_eq!(got.status, "active");

    let agents = storage.list_agents().unwrap();
    assert_eq!(agents.len(), 1);

    storage.update_agent_status("agent-1", "inactive").unwrap();
    let got = storage.get_agent("agent-1").unwrap().unwrap();
    assert_eq!(got.status, "inactive");

    let task = TaskRecord {
        id: "task-1".to_string(),
        user_input: "hello".to_string(),
        plan_json: "{}".to_string(),
        status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
    };
    storage.insert_task(&task).unwrap();
    let tasks = storage.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);

    let subtask = SubtaskRecord {
        id: "sub-1".to_string(),
        task_id: "task-1".to_string(),
        agent_id: "agent-1".to_string(),
        action: "chat".to_string(),
        params_json: "{}".to_string(),
        status: "pending".to_string(),
        result_json: None,
        started_at: None,
        finished_at: None,
    };
    storage.insert_subtask(&subtask).unwrap();
    let subtasks = storage.list_subtasks("task-1").unwrap();
    assert_eq!(subtasks.len(), 1);

    let decision = DecisionRecord {
        id: "dec-1".to_string(),
        task_id: "task-1".to_string(),
        decision_level: "direct_answer".to_string(),
        action: "chat".to_string(),
        context_json: None,
        approved: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_decision(&decision).unwrap();
    let decisions = storage.list_decisions("task-1").unwrap();
    assert_eq!(decisions.len(), 1);

    let exec = ExecutionRecord {
        id: "exec-1".to_string(),
        agent_id: "agent-1".to_string(),
        task_id: "task-1".to_string(),
        action: "chat".to_string(),
        status: "completed".to_string(),
        latency_ms: Some(100),
        error_msg: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_execution(&exec).unwrap();
    let execs = storage.list_executions("task-1").unwrap();
    assert_eq!(execs.len(), 1);

    let cap = CapabilityRecord {
        id: "cap-1".to_string(),
        agent_id: "agent-1".to_string(),
        name: "chat".to_string(),
        domain: Some("general".to_string()),
        actions: r#"["chat","analyze"]"#.to_string(),
        description: Some("General chat".to_string()),
        trust_score: 70.0,
    };
    storage.insert_capability(&cap).unwrap();
    let caps = storage.list_capabilities().unwrap();
    assert_eq!(caps.len(), 1);
}

#[test]
fn test_market_storage() {
    let storage = Storage::new_in_memory().unwrap();

    let listing = Listing {
        id: "listing-test-1".to_string(),
        item_type: "tool".to_string(),
        name: "Test Tool".to_string(),
        description: "A test tool".to_string(),
        price: 0.5,
        author: "tester".to_string(),
        rating: 4.0,
        downloads: 100,
        created_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
        screenshots: "".to_string(),
        category: "general".to_string(),
    };
    storage.save_listing(&listing).unwrap();

    let got = storage.get_listing("listing-test-1").unwrap().unwrap();
    assert_eq!(got.name, "Test Tool");
    assert_eq!(got.price, 0.5);

    let listings = storage.list_listings(None).unwrap();
    assert_eq!(listings.len(), 1);

    let filtered = storage.list_listings(Some("tool")).unwrap();
    assert_eq!(filtered.len(), 1);

    let filtered_empty = storage.list_listings(Some("knowledge")).unwrap();
    assert_eq!(filtered_empty.len(), 0);

    storage
        .update_listing_rating("listing-test-1", 4.5, 101)
        .unwrap();
    let updated = storage.get_listing("listing-test-1").unwrap().unwrap();
    assert_eq!(updated.rating, 4.5);
    assert_eq!(updated.downloads, 101);

    let tx = Transaction {
        id: "tx-test-1".to_string(),
        listing_id: "listing-test-1".to_string(),
        buyer: "user-1".to_string(),
        amount: 0.5,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    storage.save_transaction(&tx).unwrap();

    let lic = License {
        id: "lic-test-1".to_string(),
        listing_id: "listing-test-1".to_string(),
        user_id: "user-1".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
        expires_at: Some(chrono::Utc::now().to_rfc3339()),
    };
    storage.save_license(&lic).unwrap();

    let user_lics = storage.get_user_licenses("user-1").unwrap();
    assert_eq!(user_lics.len(), 1);
    assert_eq!(user_lics[0].listing_id, "listing-test-1");

    let no_lics = storage.get_user_licenses("unknown").unwrap();
    assert_eq!(no_lics.len(), 0);

    storage.delete_listing("listing-test-1").unwrap();
    assert!(storage.get_listing("listing-test-1").unwrap().is_none());
}
