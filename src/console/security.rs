//! security — Renders security state for console surfaces.
use crate::core::error::MornError;
use crate::core::dual_llm::DualLlmGuard;
use crate::core::security::SecurityPolicy;

pub struct SecurityView {
    pub dual_llm: DualLlmGuard,
    pub policies: Vec<PolicyDef>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PolicyDef {
    pub name: String,
    pub level: String,
    pub pattern: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IncidentEntry {
    pub timestamp: String,
    pub input_preview: String,
    pub risk: String,
    pub blocked: bool,
}

impl SecurityView {
    pub fn new(dual_llm: DualLlmGuard, policies: Vec<PolicyDef>) -> Self {
        Self { dual_llm, policies }
    }

    pub fn render_summary(&self) -> String {
        let blocked_count = self
            .dual_llm
            .get_log()
            .iter()
            .filter(|entry| !entry.allowed)
            .count();
        let suspicious_count = self
            .dual_llm
            .get_log()
            .iter()
            .filter(|entry| entry.risk != "none")
            .count();

        format!(
            "Security Summary\npolicies: {}\nblocked: {}\nsuspicious: {}",
            self.policies.len(),
            blocked_count,
            suspicious_count
        )
    }

    pub fn render_incidents(&self) -> Vec<IncidentEntry> {
        self.dual_llm
            .get_log()
            .iter()
            .rev()
            .filter(|entry| !entry.allowed || entry.risk != "none")
            .map(|entry| IncidentEntry {
                timestamp: entry.timestamp.clone(),
                input_preview: entry.input_preview.clone(),
                risk: entry.risk.clone(),
                blocked: !entry.allowed,
            })
            .collect()
    }
}

impl From<&SecurityPolicy> for PolicyDef {
    fn from(policy: &SecurityPolicy) -> Self {
        Self {
            name: policy.name.clone(),
            level: policy.level.as_str().to_string(),
            pattern: policy.pattern.clone(),
            description: policy.description.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dual_llm::{CheckResult, DualLlmGuard};
    use crate::core::security::SecurityGuard;

    fn view_with_logs() -> SecurityView {
        let mut guard = DualLlmGuard::new(None, None);
        assert!(matches!(
            guard.inspect("DROP TABLE users", &serde_json::json!({})),
            CheckResult::Block(_)
        ));
        guard.inspect("hello", &serde_json::json!({}));
        let policies = SecurityGuard::new()
            .list_policies()
            .iter()
            .map(PolicyDef::from)
            .collect();

        SecurityView::new(guard, policies)
    }

    #[test]
    fn summary_counts_policies_and_incidents() {
        let view = view_with_logs();
        let summary = view.render_summary();

        assert!(summary.contains("policies: 13"));
        assert!(summary.contains("blocked: 1"));
        assert!(summary.contains("suspicious: 1"));
    }

    #[test]
    fn incidents_include_recent_suspicious_entries() {
        let view = view_with_logs();
        let incidents = view.render_incidents();

        assert_eq!(incidents.len(), 1);
        assert!(incidents[0].blocked);
        assert_eq!(incidents[0].risk, "high");
    }

    #[test]
    fn policy_def_maps_core_policy_fields() {
        let guard = SecurityGuard::new();
        let policy = PolicyDef::from(guard.get_policy("format_disk").unwrap());

        assert_eq!(policy.name, "format_disk");
        assert_eq!(policy.level, "L1HardBlocked");
        assert_eq!(policy.pattern, "format_disk");
    }
}
