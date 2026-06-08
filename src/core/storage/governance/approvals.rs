//! Approval request storage operations.

use rusqlite::params;

use super::super::Storage;

pub type ApprovalRequestRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl Storage {
    /// Saves a pending approval request with action, level, context, and requester metadata.
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

    /// Updates an approval request response status and optional response payload.
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

    /// Fetches an approval request by id and returns `None` when no row exists.
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

    /// Lists ids for pending approval requests ordered by oldest request first.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_request_save_list_update_get() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .save_approval_request(
                "approval-test-1",
                "delete",
                "L5",
                Some("{}"),
                Some("user-test-1"),
            )
            .unwrap();

        assert_eq!(
            storage.list_pending_approvals().unwrap(),
            vec!["approval-test-1"]
        );

        storage
            .update_approval_response("approval-test-1", "approved", Some("ok"))
            .unwrap();
        let approval = storage
            .get_approval_request("approval-test-1")
            .unwrap()
            .unwrap();
        assert_eq!(approval.3, "approved");
        assert_eq!(approval.7.as_deref(), Some("ok"));
    }
}
