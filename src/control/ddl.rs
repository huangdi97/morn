//! ddl — Control-plane table DDL (pure data; executed by store.rs init).

/// CREATE TABLE statements for all cp_* control-plane tables.
pub const CONTROL_TABLES_SQL: &str = "
            CREATE TABLE IF NOT EXISTS cp_work (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL DEFAULT 'Work',
                api_version TEXT NOT NULL,
                generation INTEGER NOT NULL DEFAULT 1,
                spec_json TEXT NOT NULL,
                status_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_work_condition (
                work_id TEXT NOT NULL,
                type TEXT NOT NULL,
                status TEXT NOT NULL,
                reason TEXT,
                observed_at TEXT,
                source_json TEXT,
                PRIMARY KEY (work_id, type)
            );

            CREATE TABLE IF NOT EXISTS cp_semantic_event (
                id TEXT PRIMARY KEY,
                work_id TEXT,
                event_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                supersedes TEXT,
                retracts TEXT,
                valid_time TEXT,
                recorded_at TEXT NOT NULL,
                provenance_json TEXT,
                contract_version TEXT,
                schema_version TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_execution_binding (
                binding_id TEXT PRIMARY KEY,
                work_ref TEXT NOT NULL,
                capability_release_digest TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                provider_version TEXT,
                runtime_ref TEXT,
                execution_environment_ref TEXT,
                authority_decision_ref TEXT,
                profile_ref TEXT,
                site_ref TEXT,
                created_at TEXT NOT NULL,
                binding_generation INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'BOUND'
            );

            CREATE TABLE IF NOT EXISTS cp_action_attempt (
                id TEXT PRIMARY KEY,
                work_ref TEXT NOT NULL,
                binding_ref TEXT NOT NULL,
                capability_ref TEXT,
                action TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                business_key TEXT,
                source_system TEXT,
                request_hash TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_receipt (
                id TEXT PRIMARY KEY,
                attempt_ref TEXT NOT NULL,
                external_ref TEXT,
                source TEXT NOT NULL,
                integrity_hash TEXT,
                recorded_at TEXT NOT NULL,
                provenance_json TEXT,
                payload_json TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_reconciliation (
                id TEXT PRIMARY KEY,
                attempt_ref TEXT NOT NULL,
                query_json TEXT,
                found TEXT,
                external_ref_found TEXT,
                result TEXT NOT NULL,
                decided_by TEXT,
                recorded_at TEXT NOT NULL,
                notes TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_observed_outcome (
                id TEXT PRIMARY KEY,
                work_ref TEXT NOT NULL,
                source TEXT NOT NULL,
                source_ref TEXT,
                valid_time TEXT,
                recorded_at TEXT NOT NULL,
                metrics_json TEXT,
                notes TEXT,
                supersedes TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_acceptance (
                id TEXT PRIMARY KEY,
                work_ref TEXT NOT NULL,
                decision TEXT NOT NULL,
                criteria_refs_json TEXT,
                outcome_refs_json TEXT,
                actor TEXT NOT NULL,
                principal TEXT,
                reason TEXT,
                decided_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cp_capability_release (
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                digest TEXT NOT NULL,
                manifest_ref TEXT,
                released_at TEXT NOT NULL,
                provenance_json TEXT,
                deprecated INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (name, version)
            );

            CREATE TABLE IF NOT EXISTS cp_qualification (
                id TEXT PRIMARY KEY,
                capability_digest TEXT NOT NULL,
                runtime_provider TEXT,
                execution_environment TEXT,
                environment_digest TEXT,
                status TEXT NOT NULL,
                version TEXT,
                qualified_at TEXT,
                evidence_json TEXT
            );

            CREATE TABLE IF NOT EXISTS cp_profile_release (
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                digest TEXT NOT NULL,
                requirements_json TEXT NOT NULL,
                released_at TEXT NOT NULL,
                PRIMARY KEY (name, version)
            );

            CREATE TABLE IF NOT EXISTS cp_outbox (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'PENDING',
                created_at TEXT NOT NULL,
                dispatched_at TEXT,
                delivery_attempts INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS cp_controller_lease (
                controller_id TEXT PRIMARY KEY,
                lease_until TEXT NOT NULL,
                checkpoint_json TEXT,
                updated_at TEXT NOT NULL
            );
";
