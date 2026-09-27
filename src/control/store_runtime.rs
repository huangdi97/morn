//! store_runtime — outbox + controller lease/checkpoint support.
//!
//! INVARIANT: the outbox is the transaction boundary between local commits and
//! external dispatch; delivery is idempotent by event_id. Leases make
//! controllers safe to run concurrently; checkpoints let them resume without
//! in-memory canonical progress.

use super::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::now_iso;
use rusqlite::{params, OptionalExtension};

/// One pending outbox delivery.
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEntry {
    pub id: String,
    pub event_id: String,
    pub event_type: String,
    pub payload_json: String,
    pub status: String,
    pub delivery_attempts: i64,
}

impl ControlPlaneStore {
    // ---- Outbox ----

    /// Enqueues an event for external delivery (idempotent by id). Call this
    /// in the SAME transaction as the local commit when atomicity matters.
    pub fn enqueue_outbox(
        &self,
        id: &str,
        event_id: &str,
        event_type: &str,
        payload_json: &str,
    ) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_outbox (id, event_id, event_type, payload_json, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'PENDING', ?5)",
                params![id, event_id, event_type, payload_json, now_iso()],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Claims the next PENDING outbox entry (marks it DISPATCHED). Idempotent:
    /// a claimed entry is not returned again.
    pub fn claim_next_outbox(&self) -> Result<Option<OutboxEntry>, MornError> {
        let conn = self.conn()?;
        let row = conn
            .query_row(
                "SELECT id, event_id, event_type, payload_json, status, delivery_attempts
                 FROM cp_outbox WHERE status = 'PENDING' ORDER BY created_at LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let Some((id, event_id, event_type, payload_json, _status, attempts)) = row else {
            return Ok(None);
        };
        conn.execute(
            "UPDATE cp_outbox SET status = 'DISPATCHED', dispatched_at = ?2, delivery_attempts = delivery_attempts + 1 WHERE id = ?1",
            params![id, now_iso()],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;

        Ok(Some(OutboxEntry {
            id,
            event_id,
            event_type,
            payload_json,
            status: "DISPATCHED".to_string(),
            delivery_attempts: attempts + 1,
        }))
    }

    /// Marks a claimed entry DONE after successful delivery.
    pub fn complete_outbox(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_outbox SET status = 'DONE' WHERE id = ?1",
            params![id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Returns a DISPATCHED entry to PENDING (redelivery after failure).
    pub fn requeue_outbox(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_outbox SET status = 'PENDING', dispatched_at = NULL WHERE id = ?1",
            params![id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // ---- Controller lease / checkpoint ----

    /// Acquires a lease for a controller (returns false when another holder
    /// holds an unexpired lease). Lease expiry is wall-clock based.
    pub fn acquire_lease(
        &self,
        controller_id: &str,
        lease_until: &str,
        checkpoint_json: Option<&str>,
    ) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let now = now_iso();
        let existing: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT lease_until, checkpoint_json FROM cp_controller_lease WHERE controller_id = ?1",
                params![controller_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;

        match existing {
            Some((until, _checkpoint)) if until > now => Ok(false),
            _ => {
                conn.execute(
                    "INSERT INTO cp_controller_lease (controller_id, lease_until, checkpoint_json, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(controller_id) DO UPDATE SET
                        lease_until = excluded.lease_until,
                        checkpoint_json = excluded.checkpoint_json,
                        updated_at = excluded.updated_at",
                    params![controller_id, lease_until, checkpoint_json, now],
                )
                .map_err(|e| MornError::Internal(e.to_string()))?;
                Ok(true)
            }
        }
    }

    /// Saves a controller checkpoint (progress survives crashes).
    pub fn save_checkpoint(
        &self,
        controller_id: &str,
        checkpoint_json: &str,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_controller_lease SET checkpoint_json = ?2, updated_at = ?3 WHERE controller_id = ?1",
            params![controller_id, checkpoint_json, now_iso()],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Loads a controller checkpoint.
    pub fn get_checkpoint(&self, controller_id: &str) -> Result<Option<String>, MornError> {
        let conn = self.conn()?;
        let checkpoint: Option<String> = conn
            .query_row(
                "SELECT checkpoint_json FROM cp_controller_lease WHERE controller_id = ?1",
                params![controller_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(checkpoint)
    }

    /// Releases a lease (controller finished its reconcile pass).
    pub fn release_lease(&self, controller_id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM cp_controller_lease WHERE controller_id = ?1",
            params![controller_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}
