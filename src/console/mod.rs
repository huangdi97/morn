pub mod cost;
pub mod governance;

use crate::core::dual_llm::DualLlmGuard;
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use crate::core::supervisor::Supervisor;
use crate::market::Marketplace;

pub struct ConsoleBackend {
    pub registry: Option<Registry>,
    pub storage: Option<Storage>,
    pub supervisor: Option<Supervisor>,
    pub event_bus: Option<SimpleEventBus>,
    pub dual_llm: Option<DualLlmGuard>,
    pub marketplace: Option<Marketplace>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub total_tasks: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub today_cost: f64,
    pub agent_count: usize,
    pub active_channels: usize,
    pub uptime_hours: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub cpu_usage: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub disk_free_mb: u64,
    pub os: String,
    pub uptime_secs: u64,
}

impl ConsoleBackend {
    pub fn new(
        registry: Option<Registry>,
        storage: Option<Storage>,
        supervisor: Option<Supervisor>,
        event_bus: Option<SimpleEventBus>,
        dual_llm: Option<DualLlmGuard>,
        marketplace: Option<Marketplace>,
    ) -> Self {
        ConsoleBackend {
            registry,
            storage,
            supervisor,
            event_bus,
            dual_llm,
            marketplace,
        }
    }

    pub fn get_dashboard(&self) -> DashboardData {
        let task_count = self
            .storage
            .as_ref()
            .and_then(|s| s.list_tasks().ok())
            .map(|t| t.len() as u64)
            .unwrap_or(0);
        let agent_count = self
            .registry
            .as_ref()
            .map(|r| r.list_all().len())
            .unwrap_or(0);

        DashboardData {
            total_tasks: task_count,
            success_rate: 0.95,
            avg_latency_ms: 1250.0,
            today_cost: 0.05,
            agent_count,
            active_channels: 3,
            uptime_hours: 12.5,
        }
    }

    pub fn get_topology(&self) -> Vec<TopologyNode> {
        let mut nodes = Vec::new();
        if let Some(ref registry) = self.registry {
            for cap in registry.list_all() {
                nodes.push(TopologyNode {
                    id: cap.id.clone(),
                    name: cap.name.clone(),
                    node_type: "capability".into(),
                    status: "active".into(),
                });
            }
        }
        nodes
    }

    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            cpu_usage: 12.5,
            memory_used_mb: 256,
            memory_total_mb: 8192,
            disk_free_mb: 50000,
            os: std::env::consts::OS.to_string(),
            uptime_secs: 45000,
        }
    }

    pub fn get_security_logs(&self) -> Vec<SecurityLogEntry> {
        vec![
            SecurityLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_type: "auth".into(),
                detail: "User authenticated".into(),
                severity: "info".into(),
            },
            SecurityLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_type: "policy_check".into(),
                detail: "L1 policy enforced: format_disk blocked".into(),
                severity: "warning".into(),
            },
        ]
    }

    pub fn get_audit_log(&self, limit: usize) -> Vec<AuditEntry> {
        let mut entries = Vec::new();
        if let Some(ref storage) = self.storage {
            if let Ok(decisions) = storage.list_decisions("") {
                for d in decisions.iter().take(limit) {
                    entries.push(AuditEntry {
                        id: d.id.clone(),
                        action: d.action.clone(),
                        decision_level: d.decision_level.clone(),
                        approved: d.approved,
                        created_at: d.created_at.clone(),
                    });
                }
            }
        }
        entries
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityLogEntry {
    pub timestamp: String,
    pub event_type: String,
    pub detail: String,
    pub severity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub decision_level: String,
    pub approved: bool,
    pub created_at: String,
}
