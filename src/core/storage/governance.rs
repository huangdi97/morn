use rusqlite::params;

use super::{ApprovalRequestRow, AuditLogRecord, CheckpointRow, DecisionRule, Storage};

impl Storage {
    pub fn get_decision_rules(
        &self,
        user_id: &str,
        keyword: &str,
    ) -> Result<Vec<DecisionRule>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at, created_at FROM decision_rules WHERE user_id = ?1 AND keyword = ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id, keyword], |row| {
                let auto_execute_int: i32 = row.get(5)?;
                Ok(DecisionRule {
                    id: Some(row.get(0)?),
                    user_id: row.get(1)?,
                    keyword: row.get(2)?,
                    level: row.get(3)?,
                    trust_threshold: row.get(4)?,
                    auto_execute: auto_execute_int != 0,
                    source: row.get(6)?,
                    hit_count: row.get(7)?,
                    last_used_at: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| e.to_string())?);
        }
        Ok(rules)
    }

    pub fn upsert_decision_rule(&self, rule: &DecisionRule) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO decision_rules (user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(user_id, keyword) DO UPDATE SET
                level = COALESCE(?3, level),
                trust_threshold = COALESCE(?4, trust_threshold),
                auto_execute = COALESCE(?5, auto_execute),
                source = COALESCE(?6, source),
                hit_count = COALESCE(?7, hit_count),
                last_used_at = COALESCE(?8, last_used_at)",
            params![
                rule.user_id, rule.keyword, rule.level, rule.trust_threshold,
                rule.auto_execute, rule.source, rule.hit_count, rule.last_used_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn increment_rule_hit(&self, rule_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE decision_rules SET hit_count = hit_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), rule_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn adjust_rule_threshold(&self, rule_id: i64, change: f64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE decision_rules SET trust_threshold = MAX(0.0, MIN(100.0, trust_threshold + ?1)) WHERE id = ?2",
            params![change, rule_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Audit Log CRUD
    pub fn insert_audit_log(&self, log: &AuditLogRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO audit_log (id, user_id, action, target_type, target_id, details_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![log.id, log.user_id, log.action, log.target_type, log.target_id, log.details_json, log.created_at],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn query_audit_log(
        &self,
        user_id: Option<&str>,
        action_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<AuditLogRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let sql = match (user_id, action_type) {
            (Some(_), Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 AND action = ?2 ORDER BY created_at DESC LIMIT ?3",
            (Some(_), None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE action = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log ORDER BY created_at DESC LIMIT ?1",
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = match (user_id, action_type) {
            (Some(uid), Some(act)) => stmt.query_map(params![uid, act, limit], map_audit_row),
            (Some(uid), None) => stmt.query_map(params![uid, limit], map_audit_row),
            (None, Some(act)) => stmt.query_map(params![act, limit], map_audit_row),
            (None, None) => stmt.query_map(params![limit], map_audit_row),
        }
        .map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(logs)
    }

    // Checkpoints CRUD
    pub fn save_checkpoint(
        &self,
        id: &str,
        session_id: &str,
        step_index: i32,
        step_name: &str,
        state_json: &str,
        metadata_json: &str,
        parent_id: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id, session_id, step_index, step_name, state_json, metadata_json,
                parent_id, chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_latest_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<Option<CheckpointRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, step_index, step_name, state_json, metadata_json, parent_id FROM checkpoints WHERE session_id = ?1 ORDER BY step_index DESC LIMIT 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some((
                row.get(0).map_err(|e| e.to_string())?,
                row.get(1).map_err(|e| e.to_string())?,
                row.get(2).map_err(|e| e.to_string())?,
                row.get(3).map_err(|e| e.to_string())?,
                row.get(4).map_err(|e| e.to_string())?,
                row.get(5).map_err(|e| e.to_string())?,
                row.get(6).map_err(|e| e.to_string())?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn list_checkpoints(
        &self,
        session_id: &str,
    ) -> Result<Vec<(String, i32, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, step_index, step_name, created_at FROM checkpoints WHERE session_id = ?1 ORDER BY step_index ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut checkpoints = Vec::new();
        for row in rows {
            checkpoints.push(row.map_err(|e| e.to_string())?);
        }
        Ok(checkpoints)
    }

    pub fn fork_checkpoint(
        &self,
        cp_id: &str,
        new_id: &str,
        new_session_id: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
             SELECT ?1, ?2, step_index, step_name, state_json, metadata_json, parent_id, ?3 FROM checkpoints WHERE id = ?4",
            params![new_id, new_session_id, chrono::Utc::now().to_rfc3339(), cp_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Approval requests CRUD
    pub fn save_approval_request(
        &self,
        id: &str,
        action: &str,
        level: &str,
        context_json: Option<&str>,
        requested_by: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO approval_requests (id, action, level, status, context_json, requested_by, created_at)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)",
            params![
                id, action, level, context_json, requested_by,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_approval_response(
        &self,
        id: &str,
        status: &str,
        response: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE approval_requests SET status = ?1, response = ?2, responded_at = ?3 WHERE id = ?4",
            params![status, response, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_approval_request(&self, id: &str) -> Result<Option<ApprovalRequestRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, action, level, status, context_json, requested_by, responded_at, response FROM approval_requests WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some((
                row.get(0).map_err(|e| e.to_string())?,
                row.get(1).map_err(|e| e.to_string())?,
                row.get(2).map_err(|e| e.to_string())?,
                row.get(3).map_err(|e| e.to_string())?,
                row.get(4).map_err(|e| e.to_string())?,
                row.get(5).map_err(|e| e.to_string())?,
                row.get(6).map_err(|e| e.to_string())?,
                row.get(7).map_err(|e| e.to_string())?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn list_pending_approvals(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id FROM approval_requests WHERE status = 'pending' ORDER BY created_at ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    // Privacy rules CRUD
    pub fn save_privacy_rule(
        &self,
        pattern: &str,
        sensitivity: &str,
        action: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO privacy_rules (pattern, sensitivity, action) VALUES (?1, ?2, ?3)",
            params![pattern, sensitivity, action],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_privacy_rules(&self) -> Result<Vec<(i64, String, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, pattern, sensitivity, action FROM privacy_rules")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| e.to_string())?);
        }
        Ok(rules)
    }
}

fn map_audit_row(row: &rusqlite::Row) -> rusqlite::Result<AuditLogRecord> {
    Ok(AuditLogRecord {
        id: row.get(0)?,
        user_id: row.get(1)?,
        action: row.get(2)?,
        target_type: row.get(3)?,
        target_id: row.get(4)?,
        details_json: row.get(5)?,
        created_at: row.get(6)?,
    })
}
