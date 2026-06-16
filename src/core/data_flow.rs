//! 数据流引擎 — 跨模块事件路由与数据传递
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FlowStatus {
    Success,
    Failed(String),
    Pending,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataFlowRecord {
    pub id: String,
    pub source: String,
    pub target: String,
    pub data_type: String,
    pub size_bytes: u64,
    pub timestamp: i64,
    pub status: FlowStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataTypeStats {
    pub count: usize,
    pub total_size_bytes: u64,
}

pub struct DataFlowLogger {
    records: Vec<DataFlowRecord>,
    max_records: usize,
}

impl DataFlowLogger {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Vec::new(),
            max_records,
        }
    }

    pub fn record_flow(
        &mut self,
        source: String,
        target: String,
        data_type: String,
        size_bytes: u64,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let record = DataFlowRecord {
            id: id.clone(),
            source,
            target,
            data_type,
            size_bytes,
            timestamp,
            status: FlowStatus::Pending,
        };

        if self.records.len() >= self.max_records {
            self.records.remove(0);
        }
        self.records.push(record);
        id
    }

    pub fn complete_flow(&mut self, id: &str) -> Option<&DataFlowRecord> {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == id) {
            record.status = FlowStatus::Success;
            Some(record)
        } else {
            None
        }
    }

    pub fn fail_flow(&mut self, id: &str, reason: String) -> Option<&DataFlowRecord> {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == id) {
            record.status = FlowStatus::Failed(reason);
            Some(record)
        } else {
            None
        }
    }

    pub fn query_by_source(&self, source: &str) -> Vec<&DataFlowRecord> {
        self.records.iter().filter(|r| r.source == source).collect()
    }

    pub fn query_by_target(&self, target: &str) -> Vec<&DataFlowRecord> {
        self.records.iter().filter(|r| r.target == target).collect()
    }

    pub fn recent_flows(&self, n: usize) -> Vec<&DataFlowRecord> {
        let mut sorted = self.records.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        sorted.truncate(n);
        sorted
    }

    pub fn stats(&self) -> HashMap<String, DataTypeStats> {
        let mut map: HashMap<String, DataTypeStats> = HashMap::new();
        for record in &self.records {
            let entry = map
                .entry(record.data_type.clone())
                .or_insert(DataTypeStats {
                    count: 0,
                    total_size_bytes: 0,
                });
            entry.count += 1;
            entry.total_size_bytes += record.size_bytes;
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_flow_creates_id() {
        let mut logger = DataFlowLogger::new(100);
        let id = logger.record_flow("src".into(), "dst".into(), "json".into(), 1024);
        assert!(!id.is_empty());
        assert!(logger.records.iter().any(|r| r.id == id));
    }

    #[test]
    fn test_complete_flow_updates_status() {
        let mut logger = DataFlowLogger::new(100);
        let id = logger.record_flow("a".into(), "b".into(), "csv".into(), 512);
        let updated = logger.complete_flow(&id);
        assert!(updated.is_some());
        assert!(matches!(updated.unwrap().status, FlowStatus::Success));
    }

    #[test]
    fn test_fail_flow_records_reason() {
        let mut logger = DataFlowLogger::new(100);
        let id = logger.record_flow("a".into(), "b".into(), "csv".into(), 512);
        let updated = logger.fail_flow(&id, "timeout".into());
        assert!(updated.is_some());
        match &updated.unwrap().status {
            FlowStatus::Failed(reason) => assert_eq!(reason, "timeout"),
            _ => panic!("expected Failed status"),
        }
    }

    #[test]
    fn test_query_by_source() {
        let mut logger = DataFlowLogger::new(100);
        logger.record_flow("s1".into(), "t1".into(), "json".into(), 10);
        logger.record_flow("s1".into(), "t2".into(), "xml".into(), 20);
        logger.record_flow("s2".into(), "t3".into(), "csv".into(), 30);
        let results = logger.query_by_source("s1");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.source == "s1"));
    }

    #[test]
    fn test_stats_by_data_type() {
        let mut logger = DataFlowLogger::new(100);
        logger.record_flow("a".into(), "b".into(), "json".into(), 100);
        logger.record_flow("c".into(), "d".into(), "json".into(), 200);
        logger.record_flow("e".into(), "f".into(), "xml".into(), 300);
        let stats = logger.stats();
        assert_eq!(stats.len(), 2);
        let json_stats = stats.get("json").unwrap();
        assert_eq!(json_stats.count, 2);
        assert_eq!(json_stats.total_size_bytes, 300);
        let xml_stats = stats.get("xml").unwrap();
        assert_eq!(xml_stats.count, 1);
        assert_eq!(xml_stats.total_size_bytes, 300);
    }
}
