//! users — Persists users, teams, permissions, and audit log data.
use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberRecord {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissionRecord {
    pub id: String,
    pub agent_id: String,
    pub user_id: String,
    pub team_id: Option<String>,
    pub permission: String,
    pub granted_at: String,
}

impl Storage {
    /// Inserts a user record and returns success when the row is stored.
    pub fn insert_user(&self, user: &UserRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO users (id, username, display_name, role, created_at, last_login)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user.id,
                user.username,
                user.display_name,
                user.role,
                user.created_at,
                user.last_login
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Fetches a user by id and returns `None` when no row exists.
    pub fn get_user(&self, id: &str) -> Result<Option<UserRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                username: row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                display_name: row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                role: row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                created_at: row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
                last_login: row.get(5).map_err(|e| MornError::Internal(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetches a user by username and returns `None` when no row exists.
    pub fn get_user_by_username(&self, username: &str) -> Result<Option<UserRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE username = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![username])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                username: row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                display_name: row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                role: row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                created_at: row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
                last_login: row.get(5).map_err(|e| MornError::Internal(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists user records ordered by newest creation time first.
    pub fn list_users(&self) -> Result<Vec<UserRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users ORDER BY created_at DESC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    role: row.get(3)?,
                    created_at: row.get(4)?,
                    last_login: row.get(5)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut users = Vec::new();
        for row in rows {
            users.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(users)
    }

    /// Updates a user's last-login timestamp by id.
    pub fn update_user_login(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE users SET last_login = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Deletes a user by id and returns success when the delete statement completes.
    pub fn delete_user(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // Teams CRUD
    /// Inserts a team record and returns success when the row is stored.
    pub fn insert_team(&self, team: &TeamRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO teams (id, name, description, owner_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                team.id,
                team.name,
                team.description,
                team.owner_id,
                team.created_at
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Fetches a team by id and returns `None` when no row exists.
    pub fn get_team(&self, id: &str) -> Result<Option<TeamRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams WHERE id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some(TeamRecord {
                id: row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                name: row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                description: row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                owner_id: row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                created_at: row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists team records ordered by newest creation time first.
    pub fn list_teams(&self) -> Result<Vec<TeamRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams ORDER BY created_at DESC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TeamRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(teams)
    }

    /// Lists teams joined by the given user id.
    pub fn list_teams_for_user(&self, user_id: &str) -> Result<Vec<TeamRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.description, t.owner_id, t.created_at
                 FROM teams t
                 INNER JOIN team_members tm ON t.id = tm.team_id
                 WHERE tm.user_id = ?1
                 ORDER BY t.created_at DESC",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![user_id], |row| {
                Ok(TeamRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(teams)
    }

    /// Updates a team's owner id and returns success when the row is updated.
    pub fn update_team_owner(&self, id: &str, new_owner_id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE teams SET owner_id = ?1 WHERE id = ?2",
            params![new_owner_id, id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Deletes a team by id and returns success when the delete statement completes.
    pub fn delete_team(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM teams WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // Team Members CRUD
    /// Inserts a team membership record and returns success when the row is stored.
    pub fn insert_team_member(&self, member: &TeamMemberRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO team_members (id, team_id, user_id, role, joined_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                member.id,
                member.team_id,
                member.user_id,
                member.role,
                member.joined_at
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Lists membership records for a team id.
    pub fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMemberRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, team_id, user_id, role, joined_at FROM team_members WHERE team_id = ?1",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![team_id], |row| {
                Ok(TeamMemberRecord {
                    id: row.get(0)?,
                    team_id: row.get(1)?,
                    user_id: row.get(2)?,
                    role: row.get(3)?,
                    joined_at: row.get(4)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut members = Vec::new();
        for row in rows {
            members.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(members)
    }

    /// Removes a user from a team and returns success when the delete statement completes.
    pub fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, user_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Updates a user's role in a team and returns success when the row is updated.
    pub fn update_team_member_role(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE team_members SET role = ?1 WHERE team_id = ?2 AND user_id = ?3",
            params![role, team_id, user_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // Agent Permissions CRUD
    /// Inserts an agent permission record and returns success when the row is stored.
    pub fn insert_agent_permission(&self, perm: &AgentPermissionRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO agent_permissions (id, agent_id, user_id, team_id, permission, granted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                perm.id,
                perm.agent_id,
                perm.user_id,
                perm.team_id,
                perm.permission,
                perm.granted_at
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Fetches a user's permission for an agent and returns `None` when no row exists.
    pub fn get_agent_permission(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<Option<AgentPermissionRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![agent_id, user_id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some(AgentPermissionRecord {
                id: row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                agent_id: row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                user_id: row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                team_id: row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                permission: row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
                granted_at: row.get(5).map_err(|e| MornError::Internal(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists all permission records for an agent id.
    pub fn list_agent_permissions(
        &self,
        agent_id: &str,
    ) -> Result<Vec<AgentPermissionRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![agent_id], |row| {
                Ok(AgentPermissionRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    user_id: row.get(2)?,
                    team_id: row.get(3)?,
                    permission: row.get(4)?,
                    granted_at: row.get(5)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut perms = Vec::new();
        for row in rows {
            perms.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(perms)
    }

    /// Deletes an agent permission by permission id.
    pub fn delete_agent_permission(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM agent_permissions WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Deletes all permissions for a user on an agent and returns success when complete.
    pub fn delete_agent_permissions_for_user(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2",
            params![agent_id, user_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
