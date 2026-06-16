//! sync — Coordinates device sync records and cross-instance state events.
use crate::core::error::MornError;
pub mod events;

use crate::core::storage::{Storage, SyncEventRecord};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: String,
    pub platform: String,
    pub last_sync: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncState {
    pub device_id: String,
    pub last_sync_key: String,
    pub pending_changes: Vec<SyncChange>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncChange {
    pub change_type: String,
    pub key: String,
    pub value: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SyncReport {
    pub pushed_events: usize,
    pub pulled_events: usize,
    pub applied_events: usize,
    pub persisted_state_changes: usize,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct PullResponse {
    #[serde(default)]
    events: Vec<SyncEventRecord>,
    #[serde(default)]
    devices: Vec<DeviceInfo>,
    last_sync_key: Option<String>,
}

pub struct SyncEngine {
    pub(crate) storage: Option<Arc<Mutex<Storage>>>,
    pub(crate) device_id: String,
    sync_server_url: Option<String>,
    pub(crate) devices: Vec<DeviceInfo>,
    pub(crate) state: SyncState,
    #[allow(dead_code)] /* 预留：后台同步循环状态 */ running: bool,
}

impl SyncEngine {
    pub fn new(device_id: &str, sync_server_url: Option<String>) -> Self {
        tracing::debug!("creating sync engine for device '{}'", device_id);
        SyncEngine {
            storage: None,
            device_id: device_id.to_string(),
            sync_server_url,
            devices: vec![DeviceInfo {
                device_id: device_id.to_string(),
                name: "local".to_string(),
                platform: std::env::consts::OS.to_string(),
                last_sync: chrono::Utc::now().to_rfc3339(),
            }],
            state: SyncState {
                device_id: device_id.to_string(),
                last_sync_key: String::new(),
                pending_changes: Vec::new(),
            },
            running: false,
        }
    }

    pub fn with_storage(mut self, storage: Arc<Mutex<Storage>>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn push_changes(&self) -> Result<usize, MornError> {
        tracing::debug!(
            "pushing storage sync changes for device '{}'",
            self.device_id
        );
        let storage = self
            .storage
            .as_ref()
            .ok_or("SyncEngine: no storage configured")?;
        let storage = storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        let events = storage.list_unsynced_events()?;
        let count = events.len();

        if count == 0 {
            return Ok(0);
        }

        if let Some(ref server_url) = self.sync_server_url {
            let event_ids: Vec<String> = events.iter().map(|event| event.id.clone()).collect();
            let payload = serde_json::json!({
                "device_id": self.device_id,
                "last_sync_key": self.state.last_sync_key,
                "events": events
            });
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|e| MornError::Internal(format!("Sync push HTTP client error: {}", e)))?;
            let resp = client
                .post(Self::endpoint_url(server_url, "/sync/push"))
                .json(&payload)
                .send()
                .map_err(|e| MornError::Internal(format!("Sync push error: {}", e)))?;
            if resp.status().is_success() {
                storage.mark_events_synced(&event_ids)?;
            } else {
                return Err(MornError::Internal(format!("Sync push returned status: {}", resp.status())));
            }
        }

        Ok(count)
    }

    pub fn pull_changes(&self) -> Result<Vec<SyncEventRecord>, MornError> {
        tracing::debug!(
            "pulling remote sync changes for device '{}'",
            self.device_id
        );
        let server_url = self
            .sync_server_url
            .as_ref()
            .ok_or("SyncEngine: no sync server URL configured")?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| MornError::Internal(format!("Sync pull HTTP client error: {}", e)))?;
        let mut query = vec![("device_id", self.device_id.as_str())];
        if !self.state.last_sync_key.is_empty() {
            query.push(("since", self.state.last_sync_key.as_str()));
        }
        let resp = client
            .get(Self::endpoint_url(server_url, "/sync/pull"))
            .query(&query)
            .send()
            .map_err(|e| MornError::Internal(format!("Sync pull error: {}", e)))?;
        if !resp.status().is_success() {
            return Err(MornError::Internal(format!("Sync pull returned status: {}", resp.status())));
        }
        let body = resp
            .text()
            .map_err(|e| MornError::Internal(format!("Sync pull read error: {}", e)))?;
        Self::parse_pull_events(&body)
    }

    pub fn push_local_change(&mut self, change_type: &str, key: &str, value: serde_json::Value) {
        tracing::debug!(
            "queued local sync change type='{}' key='{}'",
            change_type,
            key
        );
        self.state.pending_changes.push(SyncChange {
            change_type: change_type.to_string(),
            key: key.to_string(),
            value,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn pending_changes(&self) -> &[SyncChange] {
        &self.state.pending_changes
    }

    pub fn sync_local_state(&mut self) -> Result<usize, MornError> {
        let count = self.state.pending_changes.len();
        let now = chrono::Utc::now().to_rfc3339();
        tracing::info!("syncing {} pending local change(s)", count);
        if let Some(storage) = &self.storage {
            let storage = storage.lock().map_err(|e| MornError::Internal(e.to_string()))?;
            for change in &self.state.pending_changes {
                let event = SyncEventRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    entity_type: "state".to_string(),
                    entity_id: change.key.clone(),
                    action: change.change_type.clone(),
                    data_json: serde_json::to_string(&change.value)
                        .map_err(|e| MornError::Internal(format!("Sync state encode error: {}", e)))?,
                    timestamp: change.timestamp.clone(),
                    device_id: self.device_id.clone(),
                    synced: false,
                };
                storage.insert_sync_event(&event)?;
            }
        }
        self.state.last_sync_key = now.clone();
        self.touch_device(&self.device_id.clone(), now);
        self.state.pending_changes.clear();
        Ok(count)
    }

    pub fn sync_once(&mut self) -> Result<SyncReport, MornError> {
        let persisted_state_changes = self.sync_local_state()?;
        let pushed_events = self.push_changes()?;
        let pulled = if self.sync_server_url.is_some() {
            self.pull_changes()?
        } else {
            Vec::new()
        };
        let pulled_events = pulled.len();
        let applied_events = if pulled.is_empty() {
            0
        } else {
            self.apply_remote_changes(&pulled)?
        };

        Ok(SyncReport {
            pushed_events,
            pulled_events,
            applied_events,
            persisted_state_changes,
        })
    }

    pub fn state(&self) -> &SyncState {
        &self.state
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    pub(crate) fn touch_device(&mut self, device_id: &str, last_sync: String) {
        if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == device_id) {
            device.last_sync = last_sync;
        }
    }
}

pub fn start_sync_loop(engine: Arc<Mutex<SyncEngine>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        if let Ok(mut engine) = engine.lock() {
            if let Err(e) = engine.sync_once() {
                tracing::warn!("sync_once failed: {}", e);
            }
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

    #[test]
    fn sync_engine_tracks_pending_local_changes() {
        let mut engine = SyncEngine::new("device-1", None);

        engine.push_local_change("update", "agent-1", serde_json::json!({"name": "Agent 1"}));

        assert_eq!(engine.pending_changes().len(), 1);
    }

    #[test]
    fn sync_local_state_clears_pending_changes() {
        let storage = Storage::new_in_memory().unwrap();
        let mut engine =
            SyncEngine::new("device-1", None).with_storage(Arc::new(Mutex::new(storage.clone())));
        engine.push_local_change("update", "key-1", serde_json::json!("val"));

        let count = engine.sync_local_state().unwrap();

        assert_eq!(count, 1);
        assert!(engine.pending_changes().is_empty());
        let unsynced = storage.list_unsynced_events().unwrap();
        assert_eq!(unsynced.len(), 1);
        assert_eq!(unsynced[0].entity_type, "state");
        assert_eq!(unsynced[0].entity_id, "key-1");
    }

    #[test]
    fn apply_remote_changes_is_idempotent() {
        let storage = Storage::new_in_memory().unwrap();
        let mut engine =
            SyncEngine::new("device-1", None).with_storage(Arc::new(Mutex::new(storage.clone())));
        let event = SyncEventRecord {
            id: "remote-event-1".into(),
            entity_type: "state".into(),
            entity_id: "key-1".into(),
            action: "update".into(),
            data_json: r#"{"value":1}"#.into(),
            timestamp: "2024-01-15T12:00:00Z".into(),
            device_id: "device-2".into(),
            synced: true,
        };

        assert_eq!(engine.apply_remote_changes(&[event.clone()]).unwrap(), 1);
        assert_eq!(engine.apply_remote_changes(&[event]).unwrap(), 0);
        assert!(storage.list_unsynced_events().unwrap().is_empty());
        assert!(storage.get_sync_event("remote-event-1").unwrap().is_some());
        assert!(engine.devices().iter().any(|d| d.device_id == "device-2"));
    }

    #[test]
    fn sync_once_persists_pending_state_without_server() {
        let storage = Storage::new_in_memory().unwrap();
        let mut engine =
            SyncEngine::new("device-1", None).with_storage(Arc::new(Mutex::new(storage.clone())));
        engine.push_local_change("update", "key-1", serde_json::json!({"enabled": true}));

        let report = engine.sync_once().unwrap();

        assert_eq!(report.persisted_state_changes, 1);
        assert_eq!(report.pushed_events, 1);
        assert_eq!(report.pulled_events, 0);
        assert_eq!(storage.list_unsynced_events().unwrap().len(), 1);
    }
}
