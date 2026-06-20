//! sync — Persists device sync metadata and synchronization events.
use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub data_json: String,
    pub timestamp: String,
    pub device_id: String,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub id: String,
    pub name: String,
    pub last_seen: String,
    pub public_key: String,
}

impl Storage {
    pub fn insert_sync_event(&self, event: &SyncEventRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id, event.entity_type, event.entity_id, event.action,
                event.data_json, event.timestamp, event.device_id, event.synced as i32
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn list_unsynced_events(&self) -> Result<Vec<SyncEventRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, entity_type, entity_id, action, data_json, timestamp, device_id, synced FROM sync_events WHERE synced = 0 ORDER BY timestamp ASC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let synced_int: i32 = row.get(7)?;
                Ok(SyncEventRecord {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    data_json: row.get(4)?,
                    timestamp: row.get(5)?,
                    device_id: row.get(6)?,
                    synced: synced_int != 0,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(events)
    }

    pub fn get_sync_event(&self, id: &str) -> Result<Option<SyncEventRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, entity_type, entity_id, action, data_json, timestamp, device_id, synced FROM sync_events WHERE id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            let synced_int: i32 = row.get(7).map_err(|e| MornError::Internal(e.to_string()))?;
            Ok(Some(SyncEventRecord {
                id: row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                entity_type: row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                entity_id: row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                action: row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                data_json: row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
                timestamp: row.get(5).map_err(|e| MornError::Internal(e.to_string()))?,
                device_id: row.get(6).map_err(|e| MornError::Internal(e.to_string()))?,
                synced: synced_int != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_remote_sync_event(&self, event: &SyncEventRecord) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let changed = conn
            .execute(
                "INSERT OR IGNORE INTO sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                params![
                    event.id,
                    event.entity_type,
                    event.entity_id,
                    event.action,
                    event.data_json,
                    event.timestamp,
                    event.device_id
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(changed > 0)
    }

    pub fn mark_event_synced(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE sync_events SET synced = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn mark_events_synced(&self, ids: &[String]) -> Result<(), MornError> {
        for id in ids {
            self.mark_event_synced(id)?;
        }
        Ok(())
    }

    pub fn clear_synced_events(&self) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM sync_events WHERE synced = 1", [])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // Devices CRUD
    pub fn upsert_device(&self, device: &DeviceRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO devices (id, name, last_seen, public_key)
             VALUES (?1, ?2, ?3, ?4)",
            params![device.id, device.name, device.last_seen, device.public_key],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, name, last_seen, public_key FROM devices ORDER BY last_seen DESC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DeviceRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    last_seen: row.get(2)?,
                    public_key: row.get(3)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut devices = Vec::new();
        for row in rows {
            devices.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(devices)
    }

    pub fn delete_device(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM devices WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn list_events_since(&self, since: i64, exclude_device_id: &str) -> Result<Vec<SyncEventRecord>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, entity_type, entity_id, action, data_json, timestamp, device_id, synced FROM sync_events WHERE timestamp > ?1 AND device_id != ?2 ORDER BY timestamp ASC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![since.to_string(), exclude_device_id], |row| {
                let synced_int: i32 = row.get(7)?;
                Ok(SyncEventRecord {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    data_json: row.get(4)?,
                    timestamp: row.get(5)?,
                    device_id: row.get(6)?,
                    synced: synced_int != 0,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(events)
    }

    pub fn upsert_sync_event(&self, event: &SyncEventRecord) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let changed = conn
            .execute(
                "INSERT OR REPLACE INTO sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                params![
                    event.id,
                    event.entity_type,
                    event.entity_id,
                    event.action,
                    event.data_json,
                    event.timestamp,
                    event.device_id
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(changed > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_event_insert_list_update_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_sync_event(&SyncEventRecord {
                id: "sync-test-1".to_string(),
                entity_type: "task".to_string(),
                entity_id: "task-test-1".to_string(),
                action: "create".to_string(),
                data_json: "{}".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                device_id: "device-test-1".to_string(),
                synced: false,
            })
            .unwrap();

        assert_eq!(storage.list_unsynced_events().unwrap().len(), 1);
        assert_eq!(storage.list_unsynced_events().unwrap()[0].id, "sync-test-1");

        storage.mark_event_synced("sync-test-1").unwrap();
        assert!(storage.list_unsynced_events().unwrap().is_empty());

        storage.clear_synced_events().unwrap();
        assert_eq!(sync_event_count(&storage), 0);
    }

    #[test]
    fn device_upsert_list_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .upsert_device(&DeviceRecord {
                id: "device-test-1".to_string(),
                name: "Laptop".to_string(),
                last_seen: chrono::Utc::now().to_rfc3339(),
                public_key: "public-key".to_string(),
            })
            .unwrap();

        assert_eq!(storage.list_devices().unwrap().len(), 1);
        assert_eq!(storage.list_devices().unwrap()[0].name, "Laptop");

        storage.delete_device("device-test-1").unwrap();
        assert!(storage.list_devices().unwrap().is_empty());
    }

    fn sync_event_count(storage: &Storage) -> i64 {
        let conn = storage.conn.lock().expect("lock poisoned");
        conn.query_row("SELECT COUNT(*) FROM sync_events", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn remote_sync_event_insert_is_idempotent() {
        let storage = Storage::new_in_memory().unwrap();
        let event = SyncEventRecord {
            id: "remote-event-1".to_string(),
            entity_type: "state".to_string(),
            entity_id: "settings.theme".to_string(),
            action: "update".to_string(),
            data_json: r#"{"value":"dark"}"#.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            device_id: "remote-device".to_string(),
            synced: true,
        };

        assert!(storage.insert_remote_sync_event(&event).unwrap());
        assert!(!storage.insert_remote_sync_event(&event).unwrap());
        assert!(storage.list_unsynced_events().unwrap().is_empty());
        assert!(storage.get_sync_event("remote-event-1").unwrap().is_some());
    }
}
