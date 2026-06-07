use rusqlite::params;

use super::{AgentPermissionRecord, Storage, TeamMemberRecord, TeamRecord, UserRecord};

impl Storage {
    pub fn insert_user(&self, user: &UserRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_user(&self, id: &str) -> Result<Option<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                username: row.get(1).map_err(|e| e.to_string())?,
                display_name: row.get(2).map_err(|e| e.to_string())?,
                role: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
                last_login: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE username = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![username]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                username: row.get(1).map_err(|e| e.to_string())?,
                display_name: row.get(2).map_err(|e| e.to_string())?,
                role: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
                last_login: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_users(&self) -> Result<Vec<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut users = Vec::new();
        for row in rows {
            users.push(row.map_err(|e| e.to_string())?);
        }
        Ok(users)
    }

    pub fn update_user_login(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE users SET last_login = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_user(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Teams CRUD
    pub fn insert_team(&self, team: &TeamRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_team(&self, id: &str) -> Result<Option<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(TeamRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                description: row.get(2).map_err(|e| e.to_string())?,
                owner_id: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_teams(&self) -> Result<Vec<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| e.to_string())?);
        }
        Ok(teams)
    }

    pub fn list_teams_for_user(&self, user_id: &str) -> Result<Vec<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.description, t.owner_id, t.created_at
                 FROM teams t
                 INNER JOIN team_members tm ON t.id = tm.team_id
                 WHERE tm.user_id = ?1
                 ORDER BY t.created_at DESC",
            )
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| e.to_string())?);
        }
        Ok(teams)
    }

    pub fn update_team_owner(&self, id: &str, new_owner_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE teams SET owner_id = ?1 WHERE id = ?2",
            params![new_owner_id, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_team(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM teams WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Team Members CRUD
    pub fn insert_team_member(&self, member: &TeamMemberRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMemberRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, team_id, user_id, role, joined_at FROM team_members WHERE team_id = ?1",
            )
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut members = Vec::new();
        for row in rows {
            members.push(row.map_err(|e| e.to_string())?);
        }
        Ok(members)
    }

    pub fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_team_member_role(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE team_members SET role = ?1 WHERE team_id = ?2 AND user_id = ?3",
            params![role, team_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Agent Permissions CRUD
    pub fn insert_agent_permission(&self, perm: &AgentPermissionRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_agent_permission(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<Option<AgentPermissionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![agent_id, user_id])
            .map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(AgentPermissionRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                agent_id: row.get(1).map_err(|e| e.to_string())?,
                user_id: row.get(2).map_err(|e| e.to_string())?,
                team_id: row.get(3).map_err(|e| e.to_string())?,
                permission: row.get(4).map_err(|e| e.to_string())?,
                granted_at: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_agent_permissions(
        &self,
        agent_id: &str,
    ) -> Result<Vec<AgentPermissionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1")
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut perms = Vec::new();
        for row in rows {
            perms.push(row.map_err(|e| e.to_string())?);
        }
        Ok(perms)
    }

    pub fn delete_agent_permission(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM agent_permissions WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_agent_permissions_for_user(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2",
            params![agent_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
