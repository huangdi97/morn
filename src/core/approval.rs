//! approval — Manages approval requests, responses, and policy decisions.
use crate::core::event_stream::{EventBus, EVENT_APPROVAL_REQUESTED, EVENT_APPROVAL_RESPONDED};
use crate::core::storage::Storage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ApprovalLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ApprovalLevel {
    /// Returns the stable string identifier for this approval level.
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalLevel::Low => "low",
            ApprovalLevel::Medium => "medium",
            ApprovalLevel::High => "high",
            ApprovalLevel::Critical => "critical",
        }
    }

    /// Parses an approval level string and returns the matching level, defaulting to low.
    pub fn from_str_value(s: &str) -> Self {
        match s {
            "critical" => ApprovalLevel::Critical,
            "high" => ApprovalLevel::High,
            "medium" => ApprovalLevel::Medium,
            _ => ApprovalLevel::Low,
        }
    }

    #[allow(clippy::should_implement_trait)] /* 预留：兼容旧调用入口 */
    /// Parses an approval level string through the legacy helper entry point.
    pub fn from_str(s: &str) -> Self {
        Self::from_str_value(s)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Modified(serde_json::Value),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub action: String,
    pub level: ApprovalLevel,
    pub status: ApprovalStatus,
    pub context: serde_json::Value,
    pub requested_by: Option<String>,
    pub responded_at: Option<String>,
    pub response: Option<String>,
    pub created_at: String,
}

pub struct ApprovalManager {
    storage: Arc<Storage>,
    event_bus: Option<Arc<EventBus>>,
}

impl ApprovalManager {
    /// Creates an approval manager backed by shared storage and an optional event bus.
    pub fn new(storage: Arc<Storage>, event_bus: Option<Arc<EventBus>>) -> Self {
        ApprovalManager { storage, event_bus }
    }

    /// Creates a pending approval request for an action and returns the request record.
    pub fn request(
        &self,
        action: &str,
        level: ApprovalLevel,
        context: &serde_json::Value,
    ) -> Result<ApprovalRequest, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.storage.save_approval_request(
            &id,
            action,
            level.as_str(),
            Some(&serde_json::to_string(context).map_err(|e| e.to_string())?),
            None,
        )?;

        let request = ApprovalRequest {
            id: id.clone(),
            action: action.to_string(),
            level: level.clone(),
            status: ApprovalStatus::Pending,
            context: context.clone(),
            requested_by: None,
            responded_at: None,
            response: None,
            created_at: now,
        };

        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish_event(
                &id,
                EVENT_APPROVAL_REQUESTED,
                "approval",
                serde_json::json!({
                    "approval_id": id,
                    "action": action,
                    "level": level.as_str(),
                    "context": context,
                }),
            );
        }

        Ok(request)
    }

    /// Records a non-pending response for an approval request id.
    pub fn respond(&self, id: &str, status: ApprovalStatus) -> Result<(), String> {
        let (status_str, response_text) = match &status {
            ApprovalStatus::Approved => ("approved", None),
            ApprovalStatus::Rejected => ("rejected", None),
            ApprovalStatus::Modified(val) => ("modified", Some(val.to_string())),
            ApprovalStatus::Pending => return Err("Cannot respond with pending status".to_string()),
        };
        self.storage
            .update_approval_response(id, status_str, response_text.as_deref())?;

        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish_event(
                id,
                EVENT_APPROVAL_RESPONDED,
                "approval",
                serde_json::json!({
                    "approval_id": id,
                    "status": status_str,
                    "response": response_text,
                }),
            );
        }

        Ok(())
    }

    /// Waits for an approval response until timeout and returns the final approval status.
    pub async fn wait_for_approval(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<ApprovalStatus, String> {
        let completed = Arc::new(AtomicBool::new(false));
        let started = std::time::Instant::now();

        loop {
            if started.elapsed().as_secs() > timeout_secs {
                return Ok(ApprovalStatus::Rejected);
            }

            let row = self.storage.get_approval_request(id)?;
            if let Some((_, _, _, status_str, _, _, _, response_val)) = row {
                match status_str.as_str() {
                    "approved" => return Ok(ApprovalStatus::Approved),
                    "rejected" => return Ok(ApprovalStatus::Rejected),
                    "modified" => {
                        let val = response_val
                            .and_then(|r| serde_json::from_str(&r).ok())
                            .unwrap_or(serde_json::Value::Null);
                        return Ok(ApprovalStatus::Modified(val));
                    }
                    _ => {}
                }
            }

            if completed.load(Ordering::SeqCst) {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Err("Approval wait interrupted".to_string())
    }

    /// Scores action text for approval risk and returns a value between low and high risk ranges.
    pub fn assess_risk(&self, action: &str) -> f64 {
        let high_risk_keywords = [
            "资金",
            "transfer",
            "payment",
            "delete",
            "drop",
            "shutdown",
            "资金",
            "配置变更",
            "config",
            "password",
            "token",
        ];
        let upper = action.to_uppercase();
        for keyword in &high_risk_keywords {
            if upper.contains(&keyword.to_uppercase()) {
                return 0.85;
            }
        }

        let medium_risk_keywords = [
            "approve",
            "批准",
            "authorize",
            "授权",
            "投资",
            "commit",
            "merge",
            "deploy",
            "publish",
        ];
        for keyword in &medium_risk_keywords {
            if upper.contains(&keyword.to_uppercase()) {
                return 0.55;
            }
        }

        0.15
    }

    /// Lists ids for pending approval requests.
    pub fn list_pending(&self) -> Result<Vec<String>, String> {
        self.storage.list_pending_approvals()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn setup_manager() -> ApprovalManager {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        ApprovalManager::new(storage, None)
    }

    #[test]
    fn test_request_approval() {
        let mgr = setup_manager();
        let req = mgr
            .request(
                "delete database",
                ApprovalLevel::Critical,
                &serde_json::json!({"db": "prod"}),
            )
            .unwrap();
        assert_eq!(req.action, "delete database");
        assert_eq!(req.level, ApprovalLevel::Critical);
    }

    #[test]
    fn test_respond_approve() {
        let mgr = setup_manager();
        let req = mgr
            .request("test action", ApprovalLevel::Low, &serde_json::json!({}))
            .unwrap();
        mgr.respond(&req.id, ApprovalStatus::Approved).unwrap();
    }

    #[test]
    fn test_respond_reject() {
        let mgr = setup_manager();
        let req = mgr
            .request("test action", ApprovalLevel::Medium, &serde_json::json!({}))
            .unwrap();
        mgr.respond(&req.id, ApprovalStatus::Rejected).unwrap();
    }

    #[test]
    fn test_respond_modified() {
        let mgr = setup_manager();
        let req = mgr
            .request("test action", ApprovalLevel::High, &serde_json::json!({}))
            .unwrap();
        mgr.respond(
            &req.id,
            ApprovalStatus::Modified(serde_json::json!({"param": "changed"})),
        )
        .unwrap();
    }

    #[test]
    fn test_assess_risk_high() {
        let mgr = setup_manager();
        let risk = mgr.assess_risk("transfer funds to external account");
        assert!(risk > 0.8);
    }

    #[test]
    fn test_assess_risk_low() {
        let mgr = setup_manager();
        let risk = mgr.assess_risk("what is the weather today");
        assert!(risk < 0.5);
    }

    #[test]
    fn test_assess_risk_medium() {
        let mgr = setup_manager();
        let risk = mgr.assess_risk("approve the pull request");
        assert!(risk > 0.3 && risk < 0.8);
    }
}
