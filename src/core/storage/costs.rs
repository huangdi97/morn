use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCostRow {
    pub date: String,
    pub agent_id: String,
    pub model: String,
    pub token_count: i64,
    pub cost_usd: f64,
    pub call_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost: f64,
    pub total_calls: i64,
    pub total_tokens: i64,
    pub by_date: Vec<DailyCostRow>,
}

impl Storage {
    pub fn record_call_cost(
        &self,
        agent_id: &str,
        model: &str,
        token_count: u64,
        cost_usd: f64,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        conn.execute(
            "INSERT INTO daily_costs (date, agent_id, model, token_count, cost_usd, call_count)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)
             ON CONFLICT(date, agent_id, model) DO UPDATE SET
                token_count = token_count + ?4,
                cost_usd = cost_usd + ?5,
                call_count = call_count + 1",
            params![today, agent_id, model, token_count as i64, cost_usd],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn get_daily_costs(&self, date: &str) -> Result<Vec<DailyCostRow>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT date, agent_id, model, token_count, cost_usd, call_count
                 FROM daily_costs WHERE date = ?1",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![date], |row| {
                Ok(DailyCostRow {
                    date: row.get(0)?,
                    agent_id: row.get(1)?,
                    model: row.get(2)?,
                    token_count: row.get(3)?,
                    cost_usd: row.get(4)?,
                    call_count: row.get(5)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(result)
    }

    pub fn get_agent_costs(
        &self,
        agent_id: &str,
        days: u32,
    ) -> Result<Vec<DailyCostRow>, MornError> {
        let conn = self.conn()?;
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();
        let mut stmt = conn
            .prepare(
                "SELECT date, agent_id, model, token_count, cost_usd, call_count
                 FROM daily_costs WHERE agent_id = ?1 AND date >= ?2
                 ORDER BY date DESC",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![agent_id, cutoff], |row| {
                Ok(DailyCostRow {
                    date: row.get(0)?,
                    agent_id: row.get(1)?,
                    model: row.get(2)?,
                    token_count: row.get(3)?,
                    cost_usd: row.get(4)?,
                    call_count: row.get(5)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(result)
    }

    pub fn get_total_cost(&self, days: u32) -> Result<f64, MornError> {
        let conn = self.conn()?;
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();
        let total: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM daily_costs WHERE date >= ?1",
                params![cutoff],
                |row| row.get(0),
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(total)
    }

    pub fn get_cost_summary(&self, days: u32) -> Result<CostSummary, MornError> {
        let conn = self.conn()?;
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();

        let total_cost: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM daily_costs WHERE date >= ?1",
                params![cutoff],
                |row| row.get(0),
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let total_calls: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(call_count), 0) FROM daily_costs WHERE date >= ?1",
                params![cutoff],
                |row| row.get(0),
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let total_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(token_count), 0) FROM daily_costs WHERE date >= ?1",
                params![cutoff],
                |row| row.get(0),
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT date, agent_id, model, token_count, cost_usd, call_count
                 FROM daily_costs WHERE date >= ?1 ORDER BY date DESC",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![cutoff], |row| {
                Ok(DailyCostRow {
                    date: row.get(0)?,
                    agent_id: row.get(1)?,
                    model: row.get(2)?,
                    token_count: row.get(3)?,
                    cost_usd: row.get(4)?,
                    call_count: row.get(5)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut by_date = Vec::new();
        for row in rows {
            by_date.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }

        Ok(CostSummary {
            total_cost,
            total_calls,
            total_tokens,
            by_date,
        })
    }
}