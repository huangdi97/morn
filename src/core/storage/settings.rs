//! settings — Persists application settings and configuration values.
use rusqlite::params;

use super::Storage;

impl Storage {
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(row.get(0).map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_set_get_list_update_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage.set_setting("theme", "light").unwrap();

        assert_eq!(
            storage.get_setting("theme").unwrap().as_deref(),
            Some("light")
        );
        assert_eq!(setting_count(&storage), 1);

        storage.set_setting("theme", "dark").unwrap();
        assert_eq!(
            storage.get_setting("theme").unwrap().as_deref(),
            Some("dark")
        );

        delete_setting(&storage, "theme");
        assert!(storage.get_setting("theme").unwrap().is_none());
    }

    fn setting_count(storage: &Storage) -> i64 {
        let conn = storage.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .unwrap()
    }

    fn delete_setting(storage: &Storage, key: &str) {
        let conn = storage.conn.lock().unwrap();
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .unwrap();
    }
}
