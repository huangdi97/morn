//! oauth — Persists OAuth tokens and provider authorization metadata.
use crate::core::error::MornError;
use rusqlite::params;

use super::Storage;

pub type OAuthTokenRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub struct SaveOAuthTokenArgs<'a> {
    pub id: &'a str,
    pub provider: &'a str,
    pub user_id: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub expires_at: Option<&'a str>,
    pub scope: Option<&'a str>,
}

impl Storage {
    pub fn save_oauth_token_args(&self, args: SaveOAuthTokenArgs<'_>) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens (id, provider, user_id, access_token, refresh_token, expires_at, scope, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                args.id,
                args.provider,
                args.user_id,
                args.access_token,
                args.refresh_token,
                args.expires_at,
                args.scope,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] /* 预留：兼容既有 Storage API */
    pub fn save_oauth_token(
        &self,
        id: &str,
        provider: &str,
        user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(), MornError> {
        self.save_oauth_token_args(SaveOAuthTokenArgs {
            id,
            provider,
            user_id,
            access_token,
            refresh_token,
            expires_at,
            scope,
        })
    }

    pub fn get_oauth_token(
        &self,
        provider: &str,
        user_id: &str,
    ) -> Result<Option<OAuthTokenRow>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, provider, user_id, access_token, refresh_token, expires_at, scope FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![provider, user_id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some((
                row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(5).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(6).map_err(|e| MornError::Internal(e.to_string()))?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn delete_oauth_token(&self, provider: &str, user_id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2",
            params![provider, user_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_token_save_get_list_update_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .save_oauth_token(
                "oauth-test-1",
                "github",
                "user-test-1",
                "access-1",
                Some("refresh-1"),
                None,
                Some("repo"),
            )
            .unwrap();

        assert_eq!(
            storage
                .get_oauth_token("github", "user-test-1")
                .unwrap()
                .unwrap()
                .3,
            "access-1"
        );
        assert_eq!(token_count(&storage), 1);

        storage
            .save_oauth_token(
                "oauth-test-2",
                "github",
                "user-test-1",
                "access-2",
                None,
                None,
                Some("read"),
            )
            .unwrap();
        assert_eq!(
            storage
                .get_oauth_token("github", "user-test-1")
                .unwrap()
                .unwrap()
                .3,
            "access-2"
        );

        storage.delete_oauth_token("github", "user-test-1").unwrap();
        assert!(storage
            .get_oauth_token("github", "user-test-1")
            .unwrap()
            .is_none());
    }

    fn token_count(storage: &Storage) -> i64 {
        let conn = storage.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM oauth_tokens", [], |row| row.get(0))
            .unwrap()
    }
}
