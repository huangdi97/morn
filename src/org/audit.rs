//! audit — Records and queries organization audit log entries.
use crate::core::storage::{AuditLogRecord, Storage};

pub struct AuditLogger {
    storage: Storage,
}

impl AuditLogger {
    pub fn new(storage: Storage) -> Self {
        AuditLogger { storage }
    }

    pub fn log(
        &self,
        user_id: &str,
        action: &str,
        target_type: &str,
        target_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLogRecord, String> {
        let record = AuditLogRecord {
            id: format!("audit-{}", uuid::Uuid::new_v4()),
            user_id: user_id.to_string(),
            action: action.to_string(),
            target_type: target_type.to_string(),
            target_id: target_id.to_string(),
            details_json: details.map(|d| d.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.storage.insert_audit_log(&record)?;
        Ok(record)
    }

    pub fn log_login(&self, user_id: &str) -> Result<AuditLogRecord, String> {
        self.log(user_id, "login", "user", user_id, None)
    }

    pub fn log_logout(&self, user_id: &str) -> Result<AuditLogRecord, String> {
        self.log(user_id, "logout", "user", user_id, None)
    }

    pub fn log_agent_created(
        &self,
        user_id: &str,
        agent_id: &str,
        agent_name: &str,
    ) -> Result<AuditLogRecord, String> {
        self.log(
            user_id,
            "agent_created",
            "agent",
            agent_id,
            Some(serde_json::json!({"name": agent_name})),
        )
    }

    pub fn log_agent_updated(
        &self,
        user_id: &str,
        agent_id: &str,
        changes: serde_json::Value,
    ) -> Result<AuditLogRecord, String> {
        self.log(user_id, "agent_updated", "agent", agent_id, Some(changes))
    }

    pub fn log_agent_deleted(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<AuditLogRecord, String> {
        self.log(user_id, "agent_deleted", "agent", agent_id, None)
    }

    pub fn log_permission_changed(
        &self,
        user_id: &str,
        target_user_id: &str,
        agent_id: &str,
        permission: &str,
        action: &str,
    ) -> Result<AuditLogRecord, String> {
        self.log(
            user_id,
            &format!("permission_{}", action),
            "agent_permission",
            agent_id,
            Some(serde_json::json!({"target_user": target_user_id, "permission": permission})),
        )
    }

    pub fn log_team_changed(
        &self,
        user_id: &str,
        team_id: &str,
        action: &str,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLogRecord, String> {
        self.log(
            user_id,
            &format!("team_{}", action),
            "team",
            team_id,
            details,
        )
    }

    pub fn query(
        &self,
        user_id: Option<&str>,
        action_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<AuditLogRecord>, String> {
        self.storage.query_audit_log(user_id, action_type, limit)
    }

    pub fn get_storage(&self) -> &Storage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    fn setup() -> AuditLogger {
        let storage = Storage::new_in_memory().unwrap();
        AuditLogger::new(storage)
    }

    #[test]
    fn test_log_and_query() {
        let logger = setup();
        logger
            .log("user-1", "test_action", "test", "target-1", None)
            .unwrap();
        let logs = logger.query(Some("user-1"), None, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "test_action");
    }

    #[test]
    fn test_log_login() {
        let logger = setup();
        logger.log_login("user-1").unwrap();
        let logs = logger.query(Some("user-1"), Some("login"), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_log_logout() {
        let logger = setup();
        logger.log_logout("user-1").unwrap();
        let logs = logger.query(Some("user-1"), Some("logout"), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_log_agent_created() {
        let logger = setup();
        logger
            .log_agent_created("user-1", "agent-1", "Test Agent")
            .unwrap();
        let logs = logger.query(None, Some("agent_created"), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_query_limit() {
        let logger = setup();
        logger.log("user-1", "action1", "test", "t1", None).unwrap();
        logger.log("user-1", "action2", "test", "t2", None).unwrap();
        logger.log("user-2", "action3", "test", "t3", None).unwrap();
        let logs = logger.query(None, None, 2).unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_query_by_user_and_action() {
        let logger = setup();
        logger.log("user-1", "create", "agent", "a1", None).unwrap();
        logger.log("user-1", "delete", "agent", "a2", None).unwrap();
        logger.log("user-2", "create", "agent", "a3", None).unwrap();
        let logs = logger.query(Some("user-1"), Some("create"), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }
}
