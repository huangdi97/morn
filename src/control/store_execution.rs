//! store_execution — durable ops for binding / attempt.
//!
//! INVARIANT: attempts are unique by idempotency_key (UNIQUE constraint) —
//! duplicate dispatch is prevented at the persistence layer, not by memory.

use super::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{now_iso, ActionAttempt, AttemptStatus, ExecutionBinding};
use rusqlite::{params, OptionalExtension};

impl ControlPlaneStore {
    // ---- ExecutionBinding ----

    /// Inserts a Binding (idempotent by binding_id).
    pub fn insert_binding(&self, binding: &ExecutionBinding) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_execution_binding
                 (binding_id, work_ref, capability_release_digest, provider_id, provider_version,
                  runtime_ref, execution_environment_ref, authority_decision_ref, profile_ref,
                  site_ref, created_at, binding_generation, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    binding.binding_id,
                    binding.work_ref,
                    binding.capability_release_digest,
                    binding.provider_id,
                    binding.provider_version,
                    binding.runtime_ref,
                    binding.execution_environment_ref,
                    binding.authority_decision_ref,
                    binding.profile_ref,
                    binding.site_ref,
                    binding.created_at,
                    binding.binding_generation,
                    serde_json::to_string(&binding.status)
                        .map_err(|e| MornError::Internal(e.to_string()))?
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    pub fn get_binding(&self, binding_id: &str) -> Result<Option<ExecutionBinding>, MornError> {
        let conn = self.conn()?;
        let row = conn
            .query_row(
                "SELECT binding_id, work_ref, capability_release_digest, provider_id, provider_version,
                        runtime_ref, execution_environment_ref, authority_decision_ref, profile_ref,
                        site_ref, created_at, binding_generation, status
                 FROM cp_execution_binding WHERE binding_id = ?1",
                params![binding_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?, row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?, row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?, row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?, row.get::<_, String>(10)?,
                        row.get::<_, i64>(11)?, row.get::<_, String>(12)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let Some((
            binding_id,
            work_ref,
            digest,
            provider_id,
            provider_version,
            runtime_ref,
            env_ref,
            authority_ref,
            profile_ref,
            site_ref,
            created_at,
            generation,
            status_str,
        )) = row
        else {
            return Ok(None);
        };
        let status: crate::work::BindingStatus =
            serde_json::from_str(&status_str).map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(Some(ExecutionBinding {
            binding_id,
            work_ref,
            capability_release_digest: digest,
            provider_id,
            provider_version,
            runtime_ref,
            execution_environment_ref: env_ref,
            authority_decision_ref: authority_ref,
            profile_ref,
            site_ref,
            created_at,
            binding_generation: generation,
            status,
        }))
    }

    /// Updates a Binding's status (SUSPENDED on provider failure, MIGRATED on
    /// explicit migration).
    pub fn update_binding_status(
        &self,
        binding_id: &str,
        status: crate::work::BindingStatus,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_execution_binding SET status = ?2 WHERE binding_id = ?1",
            params![
                binding_id,
                serde_json::to_string(&status).map_err(|e| MornError::Internal(e.to_string()))?
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    // ---- ActionAttempt ----

    /// Inserts an Attempt (idempotent by idempotency_key via UNIQUE). Returns
    /// false when an attempt with the same key already exists.
    pub fn insert_attempt(&self, attempt: &ActionAttempt) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_action_attempt
                 (id, work_ref, binding_ref, capability_ref, action, idempotency_key, business_key,
                  source_system, request_hash, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    attempt.id,
                    attempt.work_ref,
                    attempt.binding_ref,
                    attempt.capability_ref,
                    attempt.action,
                    attempt.idempotency_key,
                    attempt.business_key,
                    attempt.source_system,
                    attempt.request_hash,
                    serde_json::to_string(&attempt.status)
                        .map_err(|e| MornError::Internal(e.to_string()))?,
                    attempt.created_at,
                    attempt.updated_at
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    pub fn get_attempt(&self, id: &str) -> Result<Option<ActionAttempt>, MornError> {
        let conn = self.conn()?;
        let row = conn
            .query_row(
                "SELECT id, work_ref, binding_ref, capability_ref, action, idempotency_key,
                        business_key, source_system, request_hash, status, created_at, updated_at
                 FROM cp_action_attempt WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let Some((
            id,
            work_ref,
            binding_ref,
            capability_ref,
            action,
            idempotency_key,
            business_key,
            source_system,
            request_hash,
            status_str,
            created_at,
            updated_at,
        )) = row
        else {
            return Ok(None);
        };
        let status: AttemptStatus =
            serde_json::from_str(&status_str).map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(Some(ActionAttempt {
            id,
            work_ref,
            binding_ref,
            capability_ref,
            action,
            idempotency_key,
            business_key,
            source_system,
            request_hash,
            status,
            created_at,
            updated_at,
        }))
    }

    /// Finds an attempt by idempotency key (used by reconciliation and by
    /// duplicate-dispatch prevention).
    pub fn find_attempt_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<ActionAttempt>, MornError> {
        let conn = self.conn()?;
        let id: Option<String> = conn
            .query_row(
                "SELECT id FROM cp_action_attempt WHERE idempotency_key = ?1",
                params![idempotency_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        match id {
            Some(id) => self.get_attempt(&id),
            None => Ok(None),
        }
    }

    /// Updates an Attempt's status (the state machine transitions).
    pub fn update_attempt_status(&self, id: &str, status: AttemptStatus) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_action_attempt SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![
                id,
                serde_json::to_string(&status).map_err(|e| MornError::Internal(e.to_string()))?,
                now_iso()
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}
