//! agents — Persists agent records, capabilities, and related ownership data.
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub config_json: Option<String>,
    pub status: String,
    pub trust_score: f64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRecord {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub domain: Option<String>,
    pub actions: String,
    pub description: Option<String>,
    pub trust_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingRecord {
    pub id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
    pub binding_type: String,
    pub config_json: Option<String>,
    pub created_at: String,
}

impl Storage {
    /// Inserts an agent record and returns success when the row is stored.
    pub fn insert_agent(&self, agent: &AgentRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agents (id, name, component_type, config_json, status, trust_score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                agent.id, agent.name, agent.component_type, agent.config_json,
                agent.status, agent.trust_score, agent.created_at, agent.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Fetches an agent by id and returns `None` when no row exists.
    pub fn get_agent(&self, id: &str) -> Result<Option<AgentRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, component_type, config_json, status, trust_score, created_at, updated_at FROM agents WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(AgentRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                component_type: row.get(2).map_err(|e| e.to_string())?,
                config_json: row.get(3).map_err(|e| e.to_string())?,
                status: row.get(4).map_err(|e| e.to_string())?,
                trust_score: row.get(5).map_err(|e| e.to_string())?,
                created_at: row.get(6).map_err(|e| e.to_string())?,
                updated_at: row.get(7).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists agent records ordered by newest creation time first.
    pub fn list_agents(&self) -> Result<Vec<AgentRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, component_type, config_json, status, trust_score, created_at, updated_at FROM agents ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AgentRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    component_type: row.get(2)?,
                    config_json: row.get(3)?,
                    status: row.get(4)?,
                    trust_score: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut agents = Vec::new();
        for row in rows {
            agents.push(row.map_err(|e| e.to_string())?);
        }
        Ok(agents)
    }

    /// Updates an agent status by id and records the update timestamp.
    pub fn update_agent_status(&self, id: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE agents SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Deletes an agent by id and returns success when the delete statement completes.
    pub fn delete_agent(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Capabilities CRUD
    /// Inserts a capability record and returns success when the row is stored.
    pub fn insert_capability(&self, cap: &CapabilityRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO capabilities (id, agent_id, name, domain, actions, description, trust_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                cap.id, cap.agent_id, cap.name, cap.domain,
                cap.actions, cap.description, cap.trust_score
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Lists all stored capability records.
    pub fn list_capabilities(&self) -> Result<Vec<CapabilityRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, name, domain, actions, description, trust_score FROM capabilities")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CapabilityRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    name: row.get(2)?,
                    domain: row.get(3)?,
                    actions: row.get(4)?,
                    description: row.get(5)?,
                    trust_score: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut caps = Vec::new();
        for row in rows {
            caps.push(row.map_err(|e| e.to_string())?);
        }
        Ok(caps)
    }

    /// Deletes a capability by id and returns success when the delete statement completes.
    pub fn delete_capability(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM capabilities WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent() -> AgentRecord {
        AgentRecord {
            id: "agent-test-1".to_string(),
            name: "Test Agent".to_string(),
            component_type: "agent".to_string(),
            config_json: Some("{}".to_string()),
            status: "active".to_string(),
            trust_score: 70.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
        }
    }

    #[test]
    fn agent_insert_get_list_update_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage.insert_agent(&test_agent()).unwrap();

        assert_eq!(
            storage.get_agent("agent-test-1").unwrap().unwrap().name,
            "Test Agent"
        );
        assert_eq!(storage.list_agents().unwrap().len(), 1);

        storage
            .update_agent_status("agent-test-1", "inactive")
            .unwrap();
        assert_eq!(
            storage.get_agent("agent-test-1").unwrap().unwrap().status,
            "inactive"
        );

        storage.delete_agent("agent-test-1").unwrap();
        assert!(storage.get_agent("agent-test-1").unwrap().is_none());
    }

    #[test]
    fn capability_insert_list_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage.insert_agent(&test_agent()).unwrap();
        storage
            .insert_capability(&CapabilityRecord {
                id: "cap-test-1".to_string(),
                agent_id: "agent-test-1".to_string(),
                name: "chat".to_string(),
                domain: Some("general".to_string()),
                actions: r#"["chat"]"#.to_string(),
                description: Some("Chat capability".to_string()),
                trust_score: 80.0,
            })
            .unwrap();

        assert_eq!(storage.list_capabilities().unwrap().len(), 1);
        assert_eq!(storage.list_capabilities().unwrap()[0].name, "chat");

        storage.delete_capability("cap-test-1").unwrap();
        assert!(storage.list_capabilities().unwrap().is_empty());
    }
}
