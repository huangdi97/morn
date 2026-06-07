//! team — Manages organization teams, members, and user records.
use crate::core::storage::{Storage, TeamMemberRecord, TeamRecord, UserRecord};

pub struct TeamManager {
    storage: Storage,
}

impl TeamManager {
    pub fn new(storage: Storage) -> Self {
        TeamManager { storage }
    }

    pub fn create_team(
        &self,
        name: &str,
        description: &str,
        owner_id: &str,
    ) -> Result<TeamRecord, String> {
        let owner = self
            .storage
            .get_user(owner_id)?
            .ok_or_else(|| format!("User {} not found", owner_id))?;

        let team = TeamRecord {
            id: format!("team-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            description: description.to_string(),
            owner_id: owner.id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.insert_team(&team)?;

        let member = TeamMemberRecord {
            id: format!("tm-{}", uuid::Uuid::new_v4()),
            team_id: team.id.clone(),
            user_id: owner.id,
            role: "owner".to_string(),
            joined_at: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.insert_team_member(&member)?;

        Ok(team)
    }

    pub fn add_member(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<TeamMemberRecord, String> {
        self.storage
            .get_team(team_id)?
            .ok_or_else(|| format!("Team {} not found", team_id))?;
        self.storage
            .get_user(user_id)?
            .ok_or_else(|| format!("User {} not found", user_id))?;

        let valid_roles = ["owner", "admin", "member"];
        if !valid_roles.contains(&role) {
            return Err(format!(
                "Invalid role: {}. Must be owner, admin, or member",
                role
            ));
        }

        let member = TeamMemberRecord {
            id: format!("tm-{}", uuid::Uuid::new_v4()),
            team_id: team_id.to_string(),
            user_id: user_id.to_string(),
            role: role.to_string(),
            joined_at: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.insert_team_member(&member)?;
        Ok(member)
    }

    pub fn remove_member(&self, team_id: &str, user_id: &str) -> Result<(), String> {
        let members = self.storage.list_team_members(team_id)?;
        let owner_count = members.iter().filter(|m| m.role == "owner").count();
        let target = members
            .iter()
            .find(|m| m.user_id == user_id)
            .ok_or_else(|| format!("User {} is not a member of team {}", user_id, team_id))?;

        if target.role == "owner" && owner_count <= 1 {
            return Err("Cannot remove the last owner from the team".to_string());
        }

        self.storage.remove_team_member(team_id, user_id)
    }

    pub fn list_teams(&self, user_id: &str) -> Result<Vec<TeamRecord>, String> {
        self.storage.list_teams_for_user(user_id)
    }

    pub fn list_members(&self, team_id: &str) -> Result<Vec<TeamMemberRecord>, String> {
        self.storage.list_team_members(team_id)
    }

    pub fn transfer_ownership(&self, team_id: &str, new_owner_id: &str) -> Result<(), String> {
        let team = self
            .storage
            .get_team(team_id)?
            .ok_or_else(|| format!("Team {} not found", team_id))?;
        let members = self.storage.list_team_members(team_id)?;

        let _new_owner = members
            .iter()
            .find(|m| m.user_id == new_owner_id)
            .ok_or_else(|| format!("User {} is not a member of team {}", new_owner_id, team_id))?;

        self.storage
            .update_team_member_role(team_id, &team.owner_id, "admin")?;
        self.storage
            .update_team_member_role(team_id, new_owner_id, "owner")?;
        self.storage.update_team_owner(team_id, new_owner_id)
    }

    pub fn delete_team(&self, team_id: &str) -> Result<(), String> {
        self.storage.delete_team(team_id)
    }
}

pub struct UserManager {
    storage: Storage,
}

impl UserManager {
    pub fn new(storage: Storage) -> Self {
        UserManager { storage }
    }

    pub fn register(
        &self,
        username: &str,
        display_name: &str,
        role: &str,
    ) -> Result<UserRecord, String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        let valid_roles = ["admin", "user", "viewer"];
        let role = if role.is_empty() { "user" } else { role };
        if !valid_roles.contains(&role) {
            return Err(format!(
                "Invalid role: {}. Must be admin, user, or viewer",
                role
            ));
        }

        if self.storage.get_user_by_username(username)?.is_some() {
            return Err(format!("Username '{}' already exists", username));
        }

        let user = UserRecord {
            id: format!("user-{}", uuid::Uuid::new_v4()),
            username: username.to_string(),
            display_name: display_name.to_string(),
            role: role.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
        };
        self.storage.insert_user(&user)?;
        Ok(user)
    }

    pub fn get_user(&self, id: &str) -> Result<Option<UserRecord>, String> {
        self.storage.get_user(id)
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<UserRecord>, String> {
        self.storage.get_user_by_username(username)
    }

    pub fn list_users(&self) -> Result<Vec<UserRecord>, String> {
        self.storage.list_users()
    }

    pub fn delete_user(&self, id: &str) -> Result<(), String> {
        self.storage.delete_user(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (TeamManager, UserManager) {
        let storage = Storage::new_in_memory().unwrap();
        let tm = TeamManager::new(storage.clone());
        let um = UserManager::new(storage);
        (tm, um)
    }

    #[test]
    fn test_user_register() {
        let (_, um) = setup();
        let user = um.register("alice", "Alice", "user").unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.role, "user");
    }

    #[test]
    fn test_user_register_duplicate() {
        let (_, um) = setup();
        um.register("alice", "Alice", "user").unwrap();
        let result = um.register("alice", "Alice2", "user");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_get() {
        let (_, um) = setup();
        let user = um.register("bob", "Bob", "admin").unwrap();
        let got = um.get_user(&user.id).unwrap().unwrap();
        assert_eq!(got.username, "bob");
    }

    #[test]
    fn test_user_list() {
        let (_, um) = setup();
        um.register("alice", "Alice", "user").unwrap();
        um.register("bob", "Bob", "admin").unwrap();
        let users = um.list_users().unwrap();
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_create_team() {
        let (tm, um) = setup();
        let user = um.register("owner", "Owner", "user").unwrap();
        let team = tm
            .create_team("Test Team", "A test team", &user.id)
            .unwrap();
        assert_eq!(team.name, "Test Team");
    }

    #[test]
    fn test_add_member() {
        let (tm, um) = setup();
        let owner = um.register("owner", "Owner", "user").unwrap();
        let member = um.register("member", "Member", "user").unwrap();
        let team = tm.create_team("Test Team", "Desc", &owner.id).unwrap();
        let tm_member = tm.add_member(&team.id, &member.id, "member").unwrap();
        assert_eq!(tm_member.role, "member");
    }

    #[test]
    fn test_remove_member() {
        let (tm, um) = setup();
        let owner = um.register("owner", "Owner", "user").unwrap();
        let member = um.register("member", "Member", "user").unwrap();
        let team = tm.create_team("Test Team", "Desc", &owner.id).unwrap();
        tm.add_member(&team.id, &member.id, "member").unwrap();
        tm.remove_member(&team.id, &member.id).unwrap();
        let members = tm.list_members(&team.id).unwrap();
        assert_eq!(members.len(), 1);
    }

    #[test]
    fn test_list_teams() {
        let (tm, um) = setup();
        let user = um.register("user", "User", "user").unwrap();
        tm.create_team("Team 1", "Desc 1", &user.id).unwrap();
        tm.create_team("Team 2", "Desc 2", &user.id).unwrap();
        let teams = tm.list_teams(&user.id).unwrap();
        assert_eq!(teams.len(), 2);
    }

    #[test]
    fn test_transfer_ownership() {
        let (tm, um) = setup();
        let owner = um.register("owner", "Owner", "user").unwrap();
        let new_owner = um.register("new_owner", "New Owner", "user").unwrap();
        let team = tm.create_team("Test", "Desc", &owner.id).unwrap();
        tm.add_member(&team.id, &new_owner.id, "admin").unwrap();
        tm.transfer_ownership(&team.id, &new_owner.id).unwrap();
        let updated = tm.list_members(&team.id).unwrap();
        let owner_member = updated.iter().find(|m| m.user_id == new_owner.id).unwrap();
        assert_eq!(owner_member.role, "owner");
    }

    #[test]
    fn test_add_member_invalid_role() {
        let (tm, um) = setup();
        let owner = um.register("owner", "Owner", "user").unwrap();
        let member = um.register("member", "Member", "user").unwrap();
        let team = tm.create_team("Test", "Desc", &owner.id).unwrap();
        let result = tm.add_member(&team.id, &member.id, "superadmin");
        assert!(result.is_err());
    }
}
