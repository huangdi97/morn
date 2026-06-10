//! market — Persists marketplace listings and install records.
use rusqlite::params;

use super::Storage;
use crate::market::{License, Listing, Transaction};

impl Storage {
    pub fn save_listing(&self, listing: &Listing) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO market_listings (id, item_type, name, description, price, author, rating, downloads, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                listing.id, listing.item_type, listing.name, listing.description,
                listing.price, listing.author, listing.rating, listing.downloads, listing.created_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_listings(&self, filter: Option<&str>) -> Result<Vec<Listing>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let sql = match filter {
            Some(_) => "SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings WHERE item_type = ?1 ORDER BY created_at DESC",
            None => "SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings ORDER BY created_at DESC",
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = if let Some(f) = filter {
            stmt.query_map(params![f], map_listing_row)
                .map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], map_listing_row)
                .map_err(|e| e.to_string())?
        };
        let mut listings = Vec::new();
        for row in rows {
            listings.push(row.map_err(|e| e.to_string())?);
        }
        Ok(listings)
    }

    pub fn get_listing(&self, id: &str) -> Result<Option<Listing>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(listing_from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn save_transaction(&self, tx: &Transaction) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO market_transactions (id, listing_id, buyer, amount, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tx.id, tx.listing_id, tx.buyer, tx.amount, tx.timestamp],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_license(&self, lic: &License) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO market_licenses (id, listing_id, user_id, granted_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                lic.id,
                lic.listing_id,
                lic.user_id,
                lic.granted_at,
                lic.expires_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_user_licenses(&self, user_id: &str) -> Result<Vec<License>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, listing_id, user_id, granted_at, expires_at FROM market_licenses WHERE user_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id], |row| {
                Ok(License {
                    id: row.get(0)?,
                    listing_id: row.get(1)?,
                    user_id: row.get(2)?,
                    granted_at: row.get(3)?,
                    expires_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut licenses = Vec::new();
        for row in rows {
            licenses.push(row.map_err(|e| e.to_string())?);
        }
        Ok(licenses)
    }

    pub fn update_listing_rating(
        &self,
        id: &str,
        rating: f64,
        downloads: u64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE market_listings SET rating = ?1, downloads = ?2 WHERE id = ?3",
            params![rating, downloads, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_listing(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM market_listings WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_transactions(&self) -> Result<Vec<Transaction>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, listing_id, buyer, amount, timestamp FROM market_transactions ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    listing_id: row.get(1)?,
                    buyer: row.get(2)?,
                    amount: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut txs = Vec::new();
        for row in rows {
            txs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(txs)
    }

    pub fn list_transactions_by_buyer(&self, buyer: &str) -> Result<Vec<Transaction>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, listing_id, buyer, amount, timestamp FROM market_transactions WHERE buyer = ?1 ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![buyer], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    listing_id: row.get(1)?,
                    buyer: row.get(2)?,
                    amount: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut txs = Vec::new();
        for row in rows {
            txs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(txs)
    }
}

fn map_listing_row(row: &rusqlite::Row) -> rusqlite::Result<Listing> {
    Ok(Listing {
        id: row.get(0)?,
        item_type: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        price: row.get(4)?,
        author: row.get(5)?,
        rating: row.get(6)?,
        downloads: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn listing_from_row(row: &rusqlite::Row) -> Result<Listing, String> {
    Ok(Listing {
        id: row.get(0).map_err(|e| e.to_string())?,
        item_type: row.get(1).map_err(|e| e.to_string())?,
        name: row.get(2).map_err(|e| e.to_string())?,
        description: row.get(3).map_err(|e| e.to_string())?,
        price: row.get(4).map_err(|e| e.to_string())?,
        author: row.get(5).map_err(|e| e.to_string())?,
        rating: row.get(6).map_err(|e| e.to_string())?,
        downloads: row.get(7).map_err(|e| e.to_string())?,
        created_at: row.get(8).map_err(|e| e.to_string())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_listing() -> Listing {
        Listing {
            id: "listing-test-1".to_string(),
            item_type: "tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test listing".to_string(),
            price: 1.5,
            author: "tester".to_string(),
            rating: 4.0,
            downloads: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn listing_save_get_list_update_delete() {
        let storage = Storage::new_in_memory().unwrap();
        storage.save_listing(&test_listing()).unwrap();

        assert_eq!(
            storage.get_listing("listing-test-1").unwrap().unwrap().name,
            "Test Tool"
        );
        assert_eq!(storage.list_listings(Some("tool")).unwrap().len(), 1);

        storage
            .update_listing_rating("listing-test-1", 4.5, 11)
            .unwrap();
        let updated = storage.get_listing("listing-test-1").unwrap().unwrap();
        assert_eq!(updated.rating, 4.5);
        assert_eq!(updated.downloads, 11);

        storage.delete_listing("listing-test-1").unwrap();
        assert!(storage.get_listing("listing-test-1").unwrap().is_none());
    }

    #[test]
    fn transaction_and_license_save_and_list() {
        let storage = Storage::new_in_memory().unwrap();
        storage.save_listing(&test_listing()).unwrap();
        storage
            .save_transaction(&Transaction {
                id: "tx-test-1".to_string(),
                listing_id: "listing-test-1".to_string(),
                buyer: "user-test-1".to_string(),
                amount: 1.5,
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap();
        storage
            .save_license(&License {
                id: "license-test-1".to_string(),
                listing_id: "listing-test-1".to_string(),
                user_id: "user-test-1".to_string(),
                granted_at: chrono::Utc::now().to_rfc3339(),
                expires_at: None,
            })
            .unwrap();

        let licenses = storage.get_user_licenses("user-test-1").unwrap();
        assert_eq!(licenses.len(), 1);
        assert_eq!(licenses[0].listing_id, "listing-test-1");
        assert!(storage.get_user_licenses("missing").unwrap().is_empty());
    }
}
