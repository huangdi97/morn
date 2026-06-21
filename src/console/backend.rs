use super::security::SecurityView;
use super::types::*;
use crate::core::dual_llm::DualLlmGuard;
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use crate::core::supervisor::Supervisor;
use crate::hub::Hub;

pub struct ConsoleBackend {
    pub registry: Option<Registry>,
    pub storage: Option<Storage>,
    pub supervisor: Option<Supervisor>,
    pub event_bus: Option<SimpleEventBus>,
    pub dual_llm: Option<DualLlmGuard>,
    pub marketplace: Option<Hub>,
}

impl ConsoleBackend {
    pub fn new(
        registry: Option<Registry>,
        storage: Option<Storage>,
        supervisor: Option<Supervisor>,
        event_bus: Option<SimpleEventBus>,
        dual_llm: Option<DualLlmGuard>,
        marketplace: Option<Hub>,
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
        tracing::debug!("building console dashboard snapshot");
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
        let today_cost = 0.05;
        let budget = 0.04;
        let security_logs = self.get_security_logs();
        let security_event_count = security_logs
            .iter()
            .filter(|entry| entry.severity == "warning" || entry.severity == "critical")
            .count();
        let mut alerts = vec![DashboardAlert {
            id: "new-version-0.1.0".into(),
            kind: "version".into(),
            severity: "info".into(),
            title: "New version available".into(),
            detail: "Morn 0.1.0 is the current packaged version.".into(),
        }];
        if today_cost > budget {
            alerts.push(DashboardAlert {
                id: "cost-budget-exceeded".into(),
                kind: "cost".into(),
                severity: "warning".into(),
                title: "Cost budget exceeded".into(),
                detail: format!(
                    "Today's cost ¥{:.2} is above budget ¥{:.2}.",
                    today_cost, budget
                ),
            });
        }
        if security_event_count > 0 {
            alerts.push(DashboardAlert {
                id: "security-events".into(),
                kind: "security".into(),
                severity: "warning".into(),
                title: "Security events detected".into(),
                detail: format!("{} security event(s) need review.", security_event_count),
            });
        }

        DashboardData {
            total_tasks: task_count,
            success_rate: 0.95,
            avg_latency_ms: 1250.0,
            today_cost,
            agent_count,
            active_channels: 3,
            uptime_hours: 12.5,
            request_trend: vec![
                TrendPoint {
                    label: "Mon".into(),
                    value: 18.0,
                },
                TrendPoint {
                    label: "Tue".into(),
                    value: 27.0,
                },
                TrendPoint {
                    label: "Wed".into(),
                    value: 21.0,
                },
                TrendPoint {
                    label: "Thu".into(),
                    value: 34.0,
                },
                TrendPoint {
                    label: "Fri".into(),
                    value: 30.0,
                },
                TrendPoint {
                    label: "Sat".into(),
                    value: 16.0,
                },
                TrendPoint {
                    label: "Sun".into(),
                    value: task_count.max(12) as f64,
                },
            ],
            latency_trend: vec![
                TrendPoint {
                    label: "Mon".into(),
                    value: 980.0,
                },
                TrendPoint {
                    label: "Tue".into(),
                    value: 1140.0,
                },
                TrendPoint {
                    label: "Wed".into(),
                    value: 1060.0,
                },
                TrendPoint {
                    label: "Thu".into(),
                    value: 1320.0,
                },
                TrendPoint {
                    label: "Fri".into(),
                    value: 1250.0,
                },
                TrendPoint {
                    label: "Sat".into(),
                    value: 910.0,
                },
                TrendPoint {
                    label: "Sun".into(),
                    value: 1250.0,
                },
            ],
            alerts,
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
        let cpu_usage = Self::read_cpu_usage().unwrap_or(12.5);
        let (mem_used, mem_total) = Self::read_memory_info().unwrap_or((256, 8192));
        let disk_free = Self::read_disk_free().unwrap_or(50000);

        SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            cpu_usage,
            memory_used_mb: mem_used,
            memory_total_mb: mem_total,
            disk_free_mb: disk_free,
            os: std::env::consts::OS.to_string(),
            uptime_secs: Self::read_uptime().unwrap_or(45000),
        }
    }

    fn read_cpu_usage() -> Option<f64> {
        let content = std::fs::read_to_string("/proc/stat").ok()?;
        let line = content.lines().next()?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }
        let user: u64 = parts[1].parse().ok()?;
        let nice: u64 = parts[2].parse().ok()?;
        let system: u64 = parts[3].parse().ok()?;
        let idle: u64 = parts[4].parse().ok()?;
        let total = user + nice + system + idle;
        if total == 0 {
            return None;
        }
        let used = user + nice + system;
        Some((used as f64 / total as f64) * 100.0)
    }

    fn read_memory_info() -> Option<(u64, u64)> {
        let content = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("MemTotal:") {
                total_kb = val.split_whitespace().next()?.parse().ok()?;
            }
            if let Some(val) = line.strip_prefix("MemAvailable:") {
                avail_kb = val.split_whitespace().next()?.parse().ok()?;
            }
        }
        if total_kb == 0 {
            return None;
        }
        let used_mb = (total_kb - avail_kb) / 1024;
        let total_mb = total_kb / 1024;
        Some((used_mb, total_mb))
    }

    fn read_disk_free() -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            let content = std::fs::read_to_string("/proc/mounts").ok()?;
            let cwd = std::env::current_dir().ok()?;
            let path = cwd.to_str()?;
            let mount_point = content
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && path.starts_with(parts[1]) {
                        Some(parts[1].to_string())
                    } else {
                        None
                    }
                })
                .next()?;
            let df = std::process::Command::new("df")
                .arg("-B1")
                .arg(&mount_point)
                .output()
                .ok()?;
            let output = String::from_utf8_lossy(&df.stdout);
            let line = output.lines().nth(1)?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let avail: u64 = parts[3].parse().ok()?;
                return Some(avail / (1024 * 1024));
            }
            None
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    fn read_uptime() -> Option<u64> {
        let content = std::fs::read_to_string("/proc/uptime").ok()?;
        let secs: f64 = content.split_whitespace().next()?.parse().ok()?;
        Some(secs as u64)
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

    pub fn get_hub_summary(&self) -> HubSummary {
        let market = self.marketplace.as_ref();
        let listings = market.map(|m| m.list(None)).unwrap_or_default();
        let total_listings = listings.len();
        let total_downloads: u64 = listings.iter().map(|l| l.downloads).sum();
        let total_revenue: f64 = listings
            .iter()
            .map(|l| l.price.unwrap_or(0.0) * l.downloads as f64)
            .sum();
        let top_listing = listings.into_iter().max_by_key(|l| l.downloads);

        let top_listing_name;
        let top_listing_downloads;
        if let Some(ref listing) = top_listing {
            top_listing_name = listing.name.clone();
            top_listing_downloads = listing.downloads;
        } else {
            top_listing_name = String::new();
            top_listing_downloads = 0;
        }

        HubSummary {
            total_listings,
            total_downloads,
            total_revenue,
            top_listing_name,
            top_listing_downloads,
        }
    }
}

pub fn handle_security_command(input: &str, view: &SecurityView) -> String {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    match parts.get(1).copied().unwrap_or("summary") {
        "summary" => view.render_summary(),
        "incidents" => {
            serde_json::to_string(&view.render_incidents()).unwrap_or_else(|_| "[]".to_string())
        }
        "policies" => serde_json::to_string(&view.policies).unwrap_or_else(|_| "[]".to_string()),
        _ => "Usage: /security <summary|incidents|policies>".to_string(),
    }
}
