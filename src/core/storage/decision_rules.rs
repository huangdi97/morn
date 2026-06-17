//! SQLite storage implementation for DecisionRuleStore.

use crate::core::error::MornError;
use crate::core::decision_rules::{parse_decision_level, DecisionRule, DecisionRuleStore};
use rusqlite::params;

use super::Storage;

impl Storage {
    pub fn init_decision_rule_store(&self) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS decision_rule_store (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                level TEXT NOT NULL,
                condition TEXT NOT NULL,
                effect TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}

fn row_to_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<DecisionRule> {
    let level_value: String = row.get(2)?;
    let level = parse_decision_level(&level_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid decision level: {}", level_value),
            )),
        )
    })?;

    Ok(DecisionRule {
        id: row.get(0)?,
        action: row.get(1)?,
        level,
        condition: row.get(3)?,
        effect: row.get(4)?,
        created_at: row.get(5)?,
    })
}

impl DecisionRuleStore for Storage {
    fn add_rule(&self, rule: DecisionRule) -> Result<(), MornError> {
        self.init_decision_rule_store()?;
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO decision_rule_store (id, action, level, condition, effect, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                rule.id,
                rule.action,
                rule.level.as_str(),
                rule.condition,
                rule.effect,
                rule.created_at
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    fn remove_rule(&self, id: &str) -> Result<(), MornError> {
        self.init_decision_rule_store()?;
        let conn = self.conn()?;
        let affected = conn
            .execute("DELETE FROM decision_rule_store WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if affected == 0 {
            return Err(MornError::Internal(format!("Rule '{}' not found", id)));
        }
        Ok(())
    }

    fn list_rules(&self) -> Result<Vec<DecisionRule>, MornError> {
        self.init_decision_rule_store()?;
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, action, level, condition, effect, created_at FROM decision_rule_store",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt.query_map([], row_to_rule).map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(rules)
    }

    fn find_rule(&self, action: &str) -> Result<Option<DecisionRule>, MornError> {
        self.init_decision_rule_store()?;
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, action, level, condition, effect, created_at FROM decision_rule_store WHERE action = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![action], row_to_rule)
            .map_err(|e| MornError::Internal(e.to_string()))?;
        match rows.next() {
            Some(Ok(rule)) => Ok(Some(rule)),
            Some(Err(e)) => Err(MornError::Internal(e.to_string())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::decision_rules::{DecisionRule, DecisionRuleStore};
    use crate::core::storage::Storage;
    use crate::core::supervisor::DecisionLevel;

    fn make_storage() -> Storage {
        Storage::new_in_memory().unwrap()
    }

    fn make_rule(id: &str, action: &str, level: DecisionLevel) -> DecisionRule {
        DecisionRule {
            id: id.into(),
            action: action.into(),
            level,
            condition: "test".into(),
            effect: "allow".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_add_and_find() {
        let s = make_storage();
        s.add_rule(make_rule("r1", "chat", DecisionLevel::L1DirectAnswer))
            .unwrap();
        let found = s.find_rule("chat").unwrap().unwrap();
        assert_eq!(found.id, "r1");
        assert_eq!(found.level, DecisionLevel::L1DirectAnswer);
    }

    #[test]
    fn test_add_and_list() {
        let s = make_storage();
        s.add_rule(make_rule("r2", "deploy", DecisionLevel::L4Team))
            .unwrap();
        s.add_rule(make_rule("r3", "search", DecisionLevel::L1DirectAnswer))
            .unwrap();
        let rules = s.list_rules().unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_remove() {
        let s = make_storage();
        s.add_rule(make_rule("r4", "test", DecisionLevel::L2SingleTool))
            .unwrap();
        s.remove_rule("r4").unwrap();
        assert!(s.find_rule("test").unwrap().is_none());
    }

    #[test]
    fn test_find_missing() {
        let s = make_storage();
        assert!(s.find_rule("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_add_replace() {
        let s = make_storage();
        s.add_rule(make_rule("r5", "same", DecisionLevel::L1DirectAnswer))
            .unwrap();
        s.add_rule(DecisionRule {
            id: "r5".into(),
            action: "same".into(),
            level: DecisionLevel::L4Team,
            condition: "updated".into(),
            effect: "block".into(),
            created_at: "2025-01-02T00:00:00Z".into(),
        })
        .unwrap();
        let found = s.find_rule("same").unwrap().unwrap();
        assert_eq!(found.level, DecisionLevel::L4Team);
    }
}
