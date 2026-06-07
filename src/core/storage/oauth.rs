use rusqlite::params;

use super::{OAuthTokenRow, Storage};

impl Storage {
    pub fn save_oauth_token(
        &self,
        id: &str,
        provider: &str,
        user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens (id, provider, user_id, access_token, refresh_token, expires_at, scope, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id, provider, user_id, access_token, refresh_token, expires_at, scope,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_oauth_token(
        &self,
        provider: &str,
        user_id: &str,
    ) -> Result<Option<OAuthTokenRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, provider, user_id, access_token, refresh_token, expires_at, scope FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![provider, user_id])
            .map_err(|e| e.to_string())?;
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

    pub fn delete_oauth_token(&self, provider: &str, user_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2",
            params![provider, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
