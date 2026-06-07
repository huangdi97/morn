//! data_flow_log — Records outbound data movement from the local machine.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFlowEntry {
    pub id: String,
    pub timestamp: String,
    pub target: String,
    pub data_type: String,
    pub data_size_bytes: u64,
    pub authorization: String,
    pub user_revocable: bool,
}

pub struct DataFlowLogger {
    log: Vec<DataFlowEntry>,
    max_entries: usize,
}

impl DataFlowLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            log: Vec::new(),
            max_entries,
        }
    }

    pub fn log_access(
        &mut self,
        target: &str,
        data_type: &str,
        size: u64,
        authorization: &str,
    ) -> DataFlowEntry {
        let entry = DataFlowEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            target: target.to_string(),
            data_type: data_type.to_string(),
            data_size_bytes: size,
            authorization: authorization.to_string(),
            user_revocable: is_user_revocable(authorization),
        };

        if self.max_entries > 0 {
            self.log.push(entry.clone());
            if self.log.len() > self.max_entries {
                let excess = self.log.len() - self.max_entries;
                self.log.drain(0..excess);
            }
        }

        entry
    }

    pub fn get_recent(&self, limit: usize) -> Vec<DataFlowEntry> {
        self.log.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_by_target(&self, target: &str) -> Vec<DataFlowEntry> {
        self.log
            .iter()
            .filter(|entry| entry.target == target)
            .cloned()
            .collect()
    }

    pub fn get_stats(&self) -> (u64, u64) {
        let total_entries = self.log.len() as u64;
        let total_bytes = self
            .log
            .iter()
            .map(|entry| entry.data_size_bytes)
            .sum::<u64>();
        (total_entries, total_bytes)
    }
}

fn is_user_revocable(authorization: &str) -> bool {
    !matches!(
        authorization.trim().to_lowercase().as_str(),
        "system" | "policy" | "mandatory"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_access_and_stats() {
        let mut logger = DataFlowLogger::new(10);
        let entry = logger.log_access("api.example.com", "prompt", 128, "user");

        assert_eq!(entry.target, "api.example.com");
        assert!(entry.user_revocable);
        assert_eq!(logger.get_stats(), (1, 128));
    }

    #[test]
    fn test_recent_limit_and_target_lookup() {
        let mut logger = DataFlowLogger::new(2);
        logger.log_access("a.example.com", "prompt", 10, "user");
        logger.log_access("b.example.com", "file", 20, "policy");
        logger.log_access("b.example.com", "metadata", 30, "user");

        let recent = logger.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].data_type, "metadata");

        let by_target = logger.get_by_target("b.example.com");
        assert_eq!(by_target.len(), 2);
        assert!(!by_target[0].user_revocable);
        assert_eq!(logger.get_stats(), (2, 50));
    }
}
