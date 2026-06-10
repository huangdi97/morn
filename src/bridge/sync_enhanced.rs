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

pub struct SyncEngine {
    devices: Vec<DeviceInfo>,
    state: SyncState,
}

impl SyncEngine {
    pub fn new(device_id: &str) -> Self {
        SyncEngine {
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
        }
    }

    pub fn register_device(&mut self, device: DeviceInfo) {
        if !self.devices.iter().any(|d| d.device_id == device.device_id) {
            self.devices.push(device);
        }
    }

    pub fn push_change(&mut self, change_type: &str, key: &str, value: serde_json::Value) {
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

    pub fn sync(&mut self) -> Result<usize, String> {
        let count = self.state.pending_changes.len();
        self.state.last_sync_key = chrono::Utc::now().to_rfc3339();
        for device in &mut self.devices {
            device.last_sync = chrono::Utc::now().to_rfc3339();
        }
        self.state.pending_changes.clear();
        Ok(count)
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_engine_initializes_with_local_device() {
        let engine = SyncEngine::new("device-1");
        assert_eq!(engine.devices().len(), 1);
        assert_eq!(engine.devices()[0].device_id, "device-1");
    }

    #[test]
    fn sync_engine_registers_remote_device() {
        let mut engine = SyncEngine::new("local");
        engine.register_device(DeviceInfo {
            device_id: "remote-1".to_string(),
            name: "Remote".to_string(),
            platform: "android".to_string(),
            last_sync: String::new(),
        });
        assert_eq!(engine.devices().len(), 2);
    }

    #[test]
    fn sync_engine_tracks_pending_changes() {
        let mut engine = SyncEngine::new("device-1");
        engine.push_change("update", "agent-1", serde_json::json!({"name": "Agent 1"}));
        assert_eq!(engine.pending_changes().len(), 1);
    }

    #[test]
    fn sync_clears_pending_changes() {
        let mut engine = SyncEngine::new("device-1");
        engine.push_change("update", "key-1", serde_json::json!("val"));
        let count = engine.sync().unwrap();
        assert_eq!(count, 1);
        assert!(engine.pending_changes().is_empty());
    }
}