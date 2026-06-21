use crate::core::error::MornError;
use rusqlite::params;
use super::Storage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledItem {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub installed_at: String,
}

impl Storage {
    pub fn upsert_installed_item(&self, item: &InstalledItem) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO installed_items (id, item_type, name, description, enabled, installed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item.id, item.item_type, item.name, item.description, item.enabled as i32, item.installed_at],
        )?;
        Ok(())
    }

    pub fn list_installed_items(&self) -> Result<Vec<InstalledItem>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT id, item_type, name, description, enabled, installed_at FROM installed_items ORDER BY item_type, name")?;
        let items = stmt.query_map([], |row| {
            Ok(InstalledItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
                installed_at: row.get(5)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    pub fn toggle_installed_item(&self, id: &str) -> Result<bool, MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE installed_items SET enabled = CASE WHEN enabled = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        let new_state: bool = conn.query_row(
            "SELECT enabled FROM installed_items WHERE id = ?1", params![id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )?;
        Ok(new_state)
    }

    pub fn uninstall_item(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM installed_items WHERE id = ?1", params![id])?;
        Ok(())
    }
}
