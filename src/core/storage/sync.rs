//! sync — Persists device sync metadata and synchronization events.
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
    pub fn insert_sync_event(&self, event: &SyncEventRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id, event.entity_type, event.entity_id, event.action,
                event.data_json, event.timestamp, event.device_id, event.synced as i32
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_unsynced_events(&self) -> Result<Vec<SyncEventRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, entity_type, entity_id, action, data_json, timestamp, device_id, synced FROM sync_events WHERE synced = 0 ORDER BY timestamp ASC")
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| e.to_string())?);
        }
        Ok(events)
    }

    pub fn mark_event_synced(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE sync_events SET synced = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_synced_events(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM sync_events WHERE synced = 1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Devices CRUD
    pub fn upsert_device(&self, device: &DeviceRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO devices (id, name, last_seen, public_key)
             VALUES (?1, ?2, ?3, ?4)",
            params![device.id, device.name, device.last_seen, device.public_key],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, last_seen, public_key FROM devices ORDER BY last_seen DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DeviceRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    last_seen: row.get(2)?,
                    public_key: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut devices = Vec::new();
        for row in rows {
            devices.push(row.map_err(|e| e.to_string())?);
        }
        Ok(devices)
    }

    pub fn delete_device(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM devices WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
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
        let conn = storage.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM sync_events", [], |row| row.get(0))
            .unwrap()
    }
}
