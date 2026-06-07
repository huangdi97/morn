use rusqlite::params;

use super::{AgentRecord, CapabilityRecord, Storage};

impl Storage {
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

    pub fn update_agent_status(&self, id: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE agents SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_agent(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Capabilities CRUD
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

    pub fn delete_capability(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM capabilities WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
