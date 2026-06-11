//! User storage tests.
use super::*;

fn test_user() -> UserRecord {
    UserRecord {
        id: "user-test-1".to_string(),
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        role: "user".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_login: None,
    }
}

fn test_team() -> TeamRecord {
    TeamRecord {
        id: "team-test-1".to_string(),
        name: "Test Team".to_string(),
        description: "A test team".to_string(),
        owner_id: "user-test-1".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn test_member() -> TeamMemberRecord {
    TeamMemberRecord {
        id: "member-test-1".to_string(),
        team_id: "team-test-1".to_string(),
        user_id: "user-test-1".to_string(),
        role: "member".to_string(),
        joined_at: chrono::Utc::now().to_rfc3339(),
    }
}

#[test]
fn insert_and_get_user() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let got = storage.get_user("user-test-1").unwrap().unwrap();
    assert_eq!(got.username, "testuser");
    assert_eq!(got.display_name, "Test User");
    assert_eq!(got.role, "user");
}

#[test]
fn get_user_not_found() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.get_user("nonexistent").unwrap().is_none());
}

#[test]
fn get_user_by_username() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let got = storage.get_user_by_username("testuser").unwrap().unwrap();
    assert_eq!(got.id, "user-test-1");
}

#[test]
fn get_user_by_username_not_found() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.get_user_by_username("nobody").unwrap().is_none());
}

#[test]
fn list_users_empty() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.list_users().unwrap().is_empty());
}

#[test]
fn list_users_returns_inserted_users() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let mut user2 = test_user();
    user2.id = "user-test-2".to_string();
    user2.username = "user2".to_string();
    storage.insert_user(&user2).unwrap();
    assert_eq!(storage.list_users().unwrap().len(), 2);
}

#[test]
fn update_user_login() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.update_user_login("user-test-1").unwrap();
    let got = storage.get_user("user-test-1").unwrap().unwrap();
    assert!(got.last_login.is_some());
}

#[test]
fn delete_user() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.delete_user("user-test-1").unwrap();
    assert!(storage.get_user("user-test-1").unwrap().is_none());
}

#[test]
fn insert_and_get_team() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    let got = storage.get_team("team-test-1").unwrap().unwrap();
    assert_eq!(got.name, "Test Team");
    assert_eq!(got.owner_id, "user-test-1");
}

#[test]
fn get_team_not_found() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.get_team("nonexistent").unwrap().is_none());
}

#[test]
fn list_teams_empty() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage.list_teams().unwrap().is_empty());
}

#[test]
fn list_teams_returns_inserted_teams() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    assert_eq!(storage.list_teams().unwrap().len(), 1);
}

#[test]
fn update_team_owner() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let mut user2 = test_user();
    user2.id = "user-test-2".to_string();
    user2.username = "user2".to_string();
    storage.insert_user(&user2).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage
        .update_team_owner("team-test-1", "user-test-2")
        .unwrap();
    let got = storage.get_team("team-test-1").unwrap().unwrap();
    assert_eq!(got.owner_id, "user-test-2");
}

#[test]
fn delete_team() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage.delete_team("team-test-1").unwrap();
    assert!(storage.get_team("team-test-1").unwrap().is_none());
}

#[test]
fn list_teams_for_user() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage.insert_team_member(&test_member()).unwrap();
    let teams = storage.list_teams_for_user("user-test-1").unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].name, "Test Team");
}

#[test]
fn add_user_to_team() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage.insert_team_member(&test_member()).unwrap();
    let members = storage.list_team_members("team-test-1").unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, "user-test-1");
}

#[test]
fn remove_team_member() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage.insert_team_member(&test_member()).unwrap();
    storage
        .remove_team_member("team-test-1", "user-test-1")
        .unwrap();
    assert!(storage.list_team_members("team-test-1").unwrap().is_empty());
}

#[test]
fn update_team_member_role() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    storage.insert_team(&test_team()).unwrap();
    storage.insert_team_member(&test_member()).unwrap();
    storage
        .update_team_member_role("team-test-1", "user-test-1", "admin")
        .unwrap();
    let members = storage.list_team_members("team-test-1").unwrap();
    assert_eq!(members[0].role, "admin");
}

#[test]
fn insert_and_get_agent_permission() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let perm = AgentPermissionRecord {
        id: "perm-test-1".to_string(),
        agent_id: "agent-test-1".to_string(),
        user_id: "user-test-1".to_string(),
        team_id: None,
        permission: "read".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_agent_permission(&perm).unwrap();
    let got = storage
        .get_agent_permission("agent-test-1", "user-test-1")
        .unwrap()
        .unwrap();
    assert_eq!(got.permission, "read");
}

#[test]
fn get_agent_permission_not_found() {
    let storage = Storage::new_in_memory().unwrap();
    assert!(storage
        .get_agent_permission("agent-x", "user-x")
        .unwrap()
        .is_none());
}

#[test]
fn list_agent_permissions() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let perm = AgentPermissionRecord {
        id: "perm-test-1".to_string(),
        agent_id: "agent-test-1".to_string(),
        user_id: "user-test-1".to_string(),
        team_id: None,
        permission: "read".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_agent_permission(&perm).unwrap();
    assert_eq!(
        storage
            .list_agent_permissions("agent-test-1")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn delete_agent_permission() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let perm = AgentPermissionRecord {
        id: "perm-test-1".to_string(),
        agent_id: "agent-test-1".to_string(),
        user_id: "user-test-1".to_string(),
        team_id: None,
        permission: "read".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_agent_permission(&perm).unwrap();
    storage.delete_agent_permission("perm-test-1").unwrap();
    assert!(storage
        .get_agent_permission("agent-test-1", "user-test-1")
        .unwrap()
        .is_none());
}

#[test]
fn delete_agent_permissions_for_user() {
    let storage = Storage::new_in_memory().unwrap();
    storage.insert_user(&test_user()).unwrap();
    let perm = AgentPermissionRecord {
        id: "perm-test-1".to_string(),
        agent_id: "agent-test-1".to_string(),
        user_id: "user-test-1".to_string(),
        team_id: None,
        permission: "read".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    storage.insert_agent_permission(&perm).unwrap();
    storage
        .delete_agent_permissions_for_user("agent-test-1", "user-test-1")
        .unwrap();
    assert!(storage
        .list_agent_permissions("agent-test-1")
        .unwrap()
        .is_empty());
}
