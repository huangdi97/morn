use rusqlite::params;

use super::{DeviceRecord, Storage, SyncEventRecord};

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
