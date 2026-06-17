//! Privacy rule storage operations.

use crate::core::error::MornError;
use rusqlite::params;

use super::super::Storage;

impl Storage {
    /// Saves a privacy rule pattern with sensitivity and action fields.
    pub fn save_privacy_rule(
        &self,
        pattern: &str,
        sensitivity: &str,
        action: &str,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO privacy_rules (pattern, sensitivity, action) VALUES (?1, ?2, ?3)",
            params![pattern, sensitivity, action],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Lists privacy rules as id, pattern, sensitivity, and action tuples.
    pub fn list_privacy_rules(&self) -> Result<Vec<(i64, String, String, String)>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, pattern, sensitivity, action FROM privacy_rules")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_rule_save_and_list() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .save_privacy_rule("secret", "private", "redact")
            .unwrap();

        let rules = storage.list_privacy_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].1, "secret");
        assert_eq!(rules[0].3, "redact");
    }
}
