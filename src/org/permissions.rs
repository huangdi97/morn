use crate::core::storage::{AgentPermissionRecord, Storage};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    Read,
    Use,
    Manage,
    Admin,
}

impl PermissionLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read" => Some(PermissionLevel::Read),
            "use" => Some(PermissionLevel::Use),
            "manage" => Some(PermissionLevel::Manage),
            "admin" => Some(PermissionLevel::Admin),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionLevel::Read => "read",
            PermissionLevel::Use => "use",
            PermissionLevel::Manage => "manage",
            PermissionLevel::Admin => "admin",
        }
    }
}

pub struct PermissionChecker {
    storage: Storage,
}

impl PermissionChecker {
    pub fn new(storage: Storage) -> Self {
        PermissionChecker { storage }
    }

    pub fn check(&self, user_id: &str, action: &str, target: &str) -> Result<bool, String> {
        let user = self
            .storage
            .get_user(user_id)?
            .ok_or_else(|| format!("User {} not found", user_id))?;

        if user.role == "admin" {
            return Ok(true);
        }

        let required_level = match action {
            "read" => PermissionLevel::Read,
            "use" | "call" => PermissionLevel::Use,
            "update" | "modify" | "configure" => PermissionLevel::Manage,
            "delete" | "transfer" | "grant" | "revoke" => PermissionLevel::Admin,
            _ => return Err(format!("Unknown action: {}", action)),
        };

        let perm = self.storage.get_agent_permission(target, user_id)?;
        match perm {
            Some(p) => {
                let granted = PermissionLevel::from_str(&p.permission)
                    .ok_or_else(|| format!("Invalid permission level: {}", p.permission))?;
                Ok(granted >= required_level)
            }
            None => {
                let teams = self.storage.list_teams_for_user(user_id)?;
                for team in &teams {
                    let team_perms = self.storage.list_agent_permissions(target)?;
                    for tp in team_perms {
                        if tp.team_id.as_deref() == Some(&team.id) {
                            if let Some(level) = PermissionLevel::from_str(&tp.permission) {
                                if level >= required_level {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
                Ok(false)
            }
        }
    }

    pub fn grant(
        &self,
        user_id: &str,
        agent_id: &str,
        permission: &str,
        team_id: Option<&str>,
    ) -> Result<AgentPermissionRecord, String> {
        let valid_perms = ["read", "use", "manage", "admin"];
        if !valid_perms.contains(&permission) {
            return Err(format!(
                "Invalid permission: {}. Must be read, use, manage, or admin",
                permission
            ));
        }

        self.storage
            .get_user(user_id)?
            .ok_or_else(|| format!("User {} not found", user_id))?;

        let perm = AgentPermissionRecord {
            id: format!("perm-{}", uuid::Uuid::new_v4()),
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
            team_id: team_id.map(|s| s.to_string()),
            permission: permission.to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.insert_agent_permission(&perm)?;
        Ok(perm)
    }

    pub fn revoke(&self, user_id: &str, agent_id: &str) -> Result<(), String> {
        self.storage
            .delete_agent_permissions_for_user(agent_id, user_id)
    }

    pub fn list_permissions(&self, agent_id: &str) -> Result<Vec<AgentPermissionRecord>, String> {
        self.storage.list_agent_permissions(agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;
    use crate::org::team::UserManager;

    fn setup() -> (PermissionChecker, UserManager) {
        let storage = Storage::new_in_memory().unwrap();
        let pc = PermissionChecker::new(storage.clone());
        let um = UserManager::new(storage);
        (pc, um)
    }

    #[test]
    fn test_permission_level_ordering() {
        assert!(PermissionLevel::Read < PermissionLevel::Use);
        assert!(PermissionLevel::Use < PermissionLevel::Manage);
        assert!(PermissionLevel::Manage < PermissionLevel::Admin);
        assert!(PermissionLevel::Read < PermissionLevel::Admin);
    }

    #[test]
    fn test_grant_and_check() {
        let (pc, um) = setup();
        let user = um.register("testuser", "Test User", "user").unwrap();
        let agent_id = "agent-1";

        pc.grant(&user.id, agent_id, "read", None).unwrap();
        assert!(pc.check(&user.id, "read", agent_id).unwrap());
        assert!(!pc.check(&user.id, "use", agent_id).unwrap());
    }

    #[test]
    fn test_grant_higher_level() {
        let (pc, um) = setup();
        let user = um.register("testuser", "Test User", "user").unwrap();
        let agent_id = "agent-1";

        pc.grant(&user.id, agent_id, "admin", None).unwrap();
        assert!(pc.check(&user.id, "read", agent_id).unwrap());
        assert!(pc.check(&user.id, "use", agent_id).unwrap());
        assert!(pc.check(&user.id, "modify", agent_id).unwrap());
        assert!(pc.check(&user.id, "delete", agent_id).unwrap());
    }

    #[test]
    fn test_revoke() {
        let (pc, um) = setup();
        let user = um.register("testuser", "Test User", "user").unwrap();
        let agent_id = "agent-1";

        pc.grant(&user.id, agent_id, "use", None).unwrap();
        assert!(pc.check(&user.id, "use", agent_id).unwrap());
        pc.revoke(&user.id, agent_id).unwrap();
        assert!(!pc.check(&user.id, "use", agent_id).unwrap());
    }

    #[test]
    fn test_list_permissions() {
        let (pc, um) = setup();
        let user1 = um.register("user1", "User 1", "user").unwrap();
        let user2 = um.register("user2", "User 2", "user").unwrap();
        let agent_id = "agent-1";

        pc.grant(&user1.id, agent_id, "read", None).unwrap();
        pc.grant(&user2.id, agent_id, "use", None).unwrap();

        let perms = pc.list_permissions(agent_id).unwrap();
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_admin_bypass() {
        let (pc, um) = setup();
        let admin = um.register("admin", "Admin", "admin").unwrap();
        assert!(pc.check(&admin.id, "delete", "any-agent").unwrap());
    }

    #[test]
    fn test_unknown_action() {
        let (pc, um) = setup();
        let user = um.register("user", "User", "user").unwrap();
        let result = pc.check(&user.id, "unknown_action", "agent-1");
        assert!(result.is_err());
    }
}
