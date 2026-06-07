//! sync — Coordinates device sync records and cross-instance state events.
use crate::core::storage::{DeviceRecord, Storage, SyncEventRecord};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SyncEngine {
    storage: Option<Arc<Mutex<Storage>>>,
    device_id: String,
    sync_server_url: Option<String>,
    #[allow(dead_code)] /* 预留：后台同步循环状态 */ running: bool,
}

impl SyncEngine {
    pub fn new(device_id: &str, sync_server_url: Option<String>) -> Self {
        SyncEngine {
            storage: None,
            device_id: device_id.to_string(),
            sync_server_url,
            running: false,
        }
    }

    pub fn with_storage(mut self, storage: Arc<Mutex<Storage>>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn push_changes(&self) -> Result<usize, String> {
        let storage = self
            .storage
            .as_ref()
            .ok_or("SyncEngine: no storage configured")?;
        let storage = storage.lock().map_err(|e| e.to_string())?;
        let events = storage.list_unsynced_events()?;
        let count = events.len();

        if let Some(ref server_url) = self.sync_server_url {
            let payload = serde_json::json!({
                "device_id": self.device_id,
                "events": events
            });
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|e| format!("Sync push HTTP client error: {}", e))?;
            let resp = client
                .post(format!("{}/sync/push", server_url))
                .json(&payload)
                .send()
                .map_err(|e| format!("Sync push error: {}", e))?;
            if resp.status().is_success() {
                for event in &events {
                    let _ = storage.mark_event_synced(&event.id);
                }
            } else {
                return Err(format!("Sync push returned status: {}", resp.status()));
            }
        }

        Ok(count)
    }

    pub fn pull_changes(&self) -> Result<Vec<SyncEventRecord>, String> {
        let server_url = self
            .sync_server_url
            .as_ref()
            .ok_or("SyncEngine: no sync server URL configured")?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Sync pull HTTP client error: {}", e))?;
        let resp = client
            .get(format!(
                "{}/sync/pull?device_id={}",
                server_url, self.device_id
            ))
            .send()
            .map_err(|e| format!("Sync pull error: {}", e))?;
        let events: Vec<SyncEventRecord> = resp
            .json()
            .map_err(|e| format!("Sync pull JSON error: {}", e))?;
        Ok(events)
    }

    pub fn resolve_conflicts(
        &self,
        local: &SyncEventRecord,
        remote: &SyncEventRecord,
    ) -> SyncEventRecord {
        if remote.timestamp >= local.timestamp {
            remote.clone()
        } else {
            local.clone()
        }
    }

    pub fn record_event(
        &self,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        data_json: &str,
    ) -> Result<(), String> {
        let storage = self
            .storage
            .as_ref()
            .ok_or("SyncEngine: no storage configured")?;
        let storage = storage.lock().map_err(|e| e.to_string())?;
        let event = SyncEventRecord {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            action: action.to_string(),
            data_json: data_json.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            device_id: self.device_id.clone(),
            synced: false,
        };
        storage.insert_sync_event(&event)
    }

    pub fn register_device(
        &self,
        storage: &Storage,
        name: &str,
        public_key: &str,
    ) -> Result<(), String> {
        let device = DeviceRecord {
            id: self.device_id.clone(),
            name: name.to_string(),
            last_seen: chrono::Utc::now().to_rfc3339(),
            public_key: public_key.to_string(),
        };
        storage.upsert_device(&device)
    }
}

pub fn start_sync_loop(engine: Arc<Mutex<SyncEngine>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        if let Ok(engine) = engine.lock() {
            let _ = engine.push_changes();
            let _ = engine.pull_changes();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn test_sync_event_record() {
        let storage = Storage::new_in_memory().unwrap();
        let engine =
            SyncEngine::new("device-1", None).with_storage(Arc::new(Mutex::new(storage.clone())));

        engine
            .record_event("agent", "agent-1", "update", r#"{"name":"test"}"#)
            .unwrap();

        let unsynced = storage.list_unsynced_events().unwrap();
        assert_eq!(unsynced.len(), 1);
        assert_eq!(unsynced[0].entity_type, "agent");
        assert_eq!(unsynced[0].action, "update");
    }

    #[test]
    fn test_resolve_conflicts_local_wins() {
        let local = SyncEventRecord {
            id: "evt-1".into(),
            entity_type: "agent".into(),
            entity_id: "agent-1".into(),
            action: "update".into(),
            data_json: r#"{"name":"local"}"#.into(),
            timestamp: "2024-01-15T12:00:00Z".into(),
            device_id: "device-1".into(),
            synced: false,
        };
        let remote = SyncEventRecord {
            id: "evt-1".into(),
            entity_type: "agent".into(),
            entity_id: "agent-1".into(),
            action: "update".into(),
            data_json: r#"{"name":"remote"}"#.into(),
            timestamp: "2024-01-15T11:00:00Z".into(),
            device_id: "device-2".into(),
            synced: false,
        };
        let engine = SyncEngine::new("device-1", None);
        let resolved = engine.resolve_conflicts(&local, &remote);
        assert_eq!(resolved.data_json, r#"{"name":"local"}"#);
    }

    #[test]
    fn test_resolve_conflicts_remote_wins() {
        let local = SyncEventRecord {
            id: "evt-1".into(),
            entity_type: "agent".into(),
            entity_id: "agent-1".into(),
            action: "update".into(),
            data_json: r#"{"name":"local"}"#.into(),
            timestamp: "2024-01-15T11:00:00Z".into(),
            device_id: "device-1".into(),
            synced: false,
        };
        let remote = SyncEventRecord {
            id: "evt-1".into(),
            entity_type: "agent".into(),
            entity_id: "agent-1".into(),
            action: "update".into(),
            data_json: r#"{"name":"remote"}"#.into(),
            timestamp: "2024-01-15T12:00:00Z".into(),
            device_id: "device-2".into(),
            synced: false,
        };
        let engine = SyncEngine::new("device-1", None);
        let resolved = engine.resolve_conflicts(&local, &remote);
        assert_eq!(resolved.data_json, r#"{"name":"remote"}"#);
    }
}
