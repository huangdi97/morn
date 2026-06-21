use crate::core::error::MornError;
use crate::core::storage::Storage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub layer: String,
    #[serde(rename = "source")]
    pub agent_id: String,
    pub key: String,
    #[serde(rename = "content")]
    pub value: String,
    pub metadata: String,
    pub priority: f64,
    pub expires_at: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl Storage {
    pub fn list_memories(
        &self,
        agent_id: Option<&str>,
        layer: Option<&str>,
        limit: u64,
    ) -> Result<Vec<MemoryEntry>, MornError> {
        let conn = self.conn()?;
        let mut sql = String::from(
            "SELECT id, layer, agent_id, key, value, metadata, priority, expires_at, created_at, updated_at FROM memory_entries WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(aid) = agent_id {
            sql.push_str(" AND agent_id = ?");
            params.push(Box::new(aid.to_string()));
        }
        if let Some(l) = layer {
            sql.push_str(" AND layer = ?");
            params.push(Box::new(l.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    layer: row.get(1)?,
                    agent_id: row.get(2)?,
                    key: row.get(3)?,
                    value: row.get(4)?,
                    metadata: row.get(5)?,
                    priority: row.get(6)?,
                    expires_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(results)
    }

    pub fn search_memories(
        &self,
        query: &str,
        agent_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MornError> {
        let conn = self.conn()?;
        let pattern = format!("%{}%", query);
        let mut sql = String::from(
            "SELECT id, layer, agent_id, key, value, metadata, priority, expires_at, created_at, updated_at FROM memory_entries WHERE (key LIKE ? OR value LIKE ?)",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
        if let Some(aid) = agent_id {
            sql.push_str(" AND agent_id = ?");
            params.push(Box::new(aid.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT 50");

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    layer: row.get(1)?,
                    agent_id: row.get(2)?,
                    key: row.get(3)?,
                    value: row.get(4)?,
                    metadata: row.get(5)?,
                    priority: row.get(6)?,
                    expires_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(results)
    }

    pub fn delete_memory(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM memory_entries WHERE id = ?", [id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn store_memory(&self, entry: &MemoryEntry) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO memory_entries (id, layer, agent_id, key, value, metadata, priority, expires_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                entry.id,
                entry.layer,
                entry.agent_id,
                entry.key,
                entry.value,
                entry.metadata,
                entry.priority,
                entry.expires_at,
                entry.created_at,
                entry.updated_at,
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn get_memory_layers(&self) -> Result<Vec<String>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT DISTINCT layer FROM memory_entries ORDER BY layer")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(results)
    }
}
