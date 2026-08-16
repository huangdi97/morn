# Morn Security

Security is enforced by tests in `crates/morn-core-tests/tests/migration_security.rs`
and `crates/morn-core-tests/tests/conformance.rs`, plus architecture guards.

## Guarantees

- **Workspace isolation** — records are filtered by workspace; cross-workspace
  artifact access is denied (`cross_workspace_artifact_leakage_denied`).
- **Secret redaction** — secrets are never serialized to audit/ledger records
  (`secret_never_serialized_to_audit`).
- **Governed writes** — only the Action Gateway can commit canonical state;
  providers/runtimes/connectors cannot write the database directly
  (`runtime_cannot_commit_world_directly`, `connector_write_requires_gateway_token`).
- **E3 approval** — irreversible actions require approval; unapproved E3 is
  denied (`e3_approval_enforced`).
- **Node identity & lease** — nodes authenticate with identity and lease;
  stale leases are recovered (`node_identity_and_lease_enforced`).
- **Pack/plugin safety** — names are validated against path traversal and
  command injection (`safe_name`); plugins declare permissions and cannot
  bypass Core policy (`path_traversal_rejected_by_pack_manifest`,
  `command_injection_boundary_in_pack_names`).
- **Audit** — append-only ledger; immutable execution receipts.

## Operational notes

- Keep secrets out of the repository, out of SQLite plaintext, out of logs,
  and out of error messages.
- Run the server bound to localhost by default; place it behind your reverse
  proxy/identity layer for remote exposure.
- Review E3 (irreversible) actions in production; they require explicit
  approval by policy.
- Connector integrations must go through the governed write path (gateway
  token) — never give connectors direct DB access.
