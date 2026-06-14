//! QR code scanning and device binding for session pairing.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ScanBindManager {
    active_sessions: HashMap<String, BindSession>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindSession {
    pub token: String,
    pub channel_type: String,
    pub status: BindStatus,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindStatus {
    Pending,
    Scanned,
    Confirmed,
    Expired,
}

impl ScanBindManager {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
        }
    }

    pub fn generate_bind_token(&mut self, channel_type: &str) -> BindSession {
        let now = Utc::now();
        let session = BindSession {
            token: Uuid::new_v4().to_string(),
            channel_type: channel_type.to_string(),
            status: BindStatus::Pending,
            created_at: now.to_rfc3339(),
            expires_at: (now + Duration::minutes(5)).to_rfc3339(),
        };

        self.active_sessions
            .insert(session.token.clone(), session.clone());
        session
    }

    pub fn poll_bind_status(&mut self, token: &str) -> BindStatus {
        let Some(session) = self.active_sessions.get_mut(token) else {
            return BindStatus::Expired;
        };

        if session.status != BindStatus::Confirmed && is_expired(&session.expires_at) {
            session.status = BindStatus::Expired;
        }

        session.status
    }

    pub fn confirm_bind(&mut self, token: &str, _config_json: &str) -> Result<(), String> {
        let session = self
            .active_sessions
            .get_mut(token)
            .ok_or_else(|| "Bind session not found".to_string())?;

        if is_expired(&session.expires_at) {
            session.status = BindStatus::Expired;
            return Err("Bind session is expired".to_string());
        }

        match session.status {
            BindStatus::Pending | BindStatus::Scanned => {
                session.status = BindStatus::Confirmed;
                Ok(())
            }
            BindStatus::Confirmed => Err("Bind session is already confirmed".to_string()),
            BindStatus::Expired => Err("Bind session is expired".to_string()),
        }
    }

    pub fn mark_scanned(&mut self, token: &str) -> Result<(), String> {
        let session = self
            .active_sessions
            .get_mut(token)
            .ok_or_else(|| "Bind session not found".to_string())?;

        if is_expired(&session.expires_at) {
            session.status = BindStatus::Expired;
            return Err("Bind session is expired".to_string());
        }

        match session.status {
            BindStatus::Pending => {
                session.status = BindStatus::Scanned;
                Ok(())
            }
            BindStatus::Scanned => Ok(()),
            BindStatus::Confirmed => Err("Bind session is already confirmed".to_string()),
            BindStatus::Expired => Err("Bind session is expired".to_string()),
        }
    }

    pub fn list_active_sessions(&self) -> Vec<&BindSession> {
        self.active_sessions
            .values()
            .filter(|session| {
                session.status != BindStatus::Expired && !is_expired(&session.expires_at)
            })
            .collect()
    }

    pub fn cleanup_expired(&mut self) {
        self.active_sessions.retain(|_, session| {
            session.status != BindStatus::Expired && !is_expired(&session.expires_at)
        });
    }
}

impl Default for ScanBindManager {
    fn default() -> Self {
        Self::new()
    }
}

fn is_expired(expires_at: &str) -> bool {
    DateTime::parse_from_rfc3339(expires_at)
        .map(|expires_at| Utc::now() > expires_at.with_timezone(&Utc))
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_creates_session_with_pending_status() {
        let mut manager = ScanBindManager::new();

        let session = manager.generate_bind_token("wechat");

        assert!(!session.token.is_empty());
        assert_eq!(session.channel_type, "wechat");
        assert_eq!(session.status, BindStatus::Pending);
        assert_eq!(
            manager.poll_bind_status(&session.token),
            BindStatus::Pending
        );
    }

    #[test]
    fn poll_returns_correct_status() {
        let mut manager = ScanBindManager::new();
        let session = manager.generate_bind_token("dingtalk");

        manager.mark_scanned(&session.token).unwrap();

        assert_eq!(
            manager.poll_bind_status(&session.token),
            BindStatus::Scanned
        );
    }

    #[test]
    fn confirm_bind_changes_status_to_confirmed() {
        let mut manager = ScanBindManager::new();
        let session = manager.generate_bind_token("telegram");

        manager
            .confirm_bind(&session.token, r#"{"bot_token":"test"}"#)
            .unwrap();

        assert_eq!(
            manager.poll_bind_status(&session.token),
            BindStatus::Confirmed
        );
    }

    #[test]
    fn poll_auto_expires_after_time() {
        let mut manager = ScanBindManager::new();
        let session = manager.generate_bind_token("feishu");
        manager
            .active_sessions
            .get_mut(&session.token)
            .unwrap()
            .expires_at = (Utc::now() - Duration::minutes(1)).to_rfc3339();

        assert_eq!(
            manager.poll_bind_status(&session.token),
            BindStatus::Expired
        );
    }

    #[test]
    fn list_active_returns_only_non_expired_sessions() {
        let mut manager = ScanBindManager::new();
        let active = manager.generate_bind_token("wechat");
        let expired = manager.generate_bind_token("qqbot");
        manager
            .active_sessions
            .get_mut(&expired.token)
            .unwrap()
            .expires_at = (Utc::now() - Duration::minutes(1)).to_rfc3339();

        let active_sessions = manager.list_active_sessions();

        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].token, active.token);
    }
}
