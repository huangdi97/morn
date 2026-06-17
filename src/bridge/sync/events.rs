//! Event handling for device sync.
use super::{DeviceInfo, PullResponse, SyncEngine};
use crate::core::error::MornError;
use crate::core::storage::{DeviceRecord, Storage, SyncEventRecord};

impl SyncEngine {
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
    ) -> Result<(), MornError> {
        tracing::debug!(
            "recording sync event entity_type='{}' entity_id='{}' action='{}'",
            entity_type,
            entity_id,
            action
        );
        let storage = self
            .storage
            .as_ref()
            .ok_or("SyncEngine: no storage configured")?;
        let storage = storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
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
    ) -> Result<(), MornError> {
        let device = DeviceRecord {
            id: self.device_id.clone(),
            name: name.to_string(),
            last_seen: chrono::Utc::now().to_rfc3339(),
            public_key: public_key.to_string(),
        };
        storage.upsert_device(&device)
    }

    pub fn register_peer_device(&mut self, device: DeviceInfo) {
        if let Some(existing) = self
            .devices
            .iter_mut()
            .find(|d| d.device_id == device.device_id)
        {
            *existing = device;
        } else {
            tracing::info!("registered sync peer device '{}'", device.device_id);
            self.devices.push(device);
        }
    }

    pub fn apply_remote_changes(&mut self, events: &[SyncEventRecord]) -> Result<usize, MornError> {
        let storage = self
            .storage
            .as_ref()
            .ok_or("SyncEngine: no storage configured")?
            .clone();

        let mut applied = 0;
        for event in events {
            if event.device_id == self.device_id {
                continue;
            }

            let inserted = {
                let storage = storage
                    .lock()
                    .map_err(|e| MornError::Internal(e.to_string()))?;
                storage.insert_remote_sync_event(event)?
            };
            if inserted {
                applied += 1;
            }

            self.register_peer_device(DeviceInfo {
                device_id: event.device_id.clone(),
                name: event.device_id.clone(),
                platform: "unknown".to_string(),
                last_sync: event.timestamp.clone(),
            });
        }

        if applied > 0 {
            self.state.last_sync_key = chrono::Utc::now().to_rfc3339();
            self.touch_device(&self.device_id.clone(), self.state.last_sync_key.clone());
        }

        Ok(applied)
    }

    pub(super) fn endpoint_url(server_url: &str, path: &str) -> String {
        format!(
            "{}/{}",
            server_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(super) fn parse_pull_events(body: &str) -> Result<Vec<SyncEventRecord>, MornError> {
        if let Ok(events) = serde_json::from_str::<Vec<SyncEventRecord>>(body) {
            return Ok(events);
        }

        let response: PullResponse = serde_json::from_str(body)
            .map_err(|e| MornError::Internal(format!("Sync pull JSON error: {}", e)))?;
        Ok(response.events)
    }
}
