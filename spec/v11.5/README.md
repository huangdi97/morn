# Morn Protocol v11.5 — language-neutral artifacts

This directory is the machine-readable publication surface for the Morn v11.5
semantic protocol. Rust types remain the reference implementation, but the
protocol is **not** defined by Rust, Cordis, DeepSeek Harness, Pi, a database or
the Morn server process.

## Contents

- `protocol.json` — published protocol manifest and semantic-slot registry.
- `morn-protocol.schema.json` — JSON Schema 2020-12 bundle for the principal
  cross-runtime records used by v11.5.
- `examples/` — canonical fixture documents consumed by the conformance tests.

The schema bundle intentionally covers the cross-runtime boundary rather than
every internal implementation type. Provider-specific state belongs behind a
provider contract.

## Compatibility rule

Published artifacts are content-addressable/versioned. A patch may fix
non-semantic mistakes only. A change that alters a field's meaning, removes an
established semantic slot or weakens a Profile guarantee requires explicit
protocol/profile evolution and reevaluation.

The textual architecture remains authoritative when a schema cannot express a
cross-record invariant. This mirrors the common standards practice that schemas
validate structure while the specification defines behavior.

## Stable v11.5 semantic slots

`Work / Capability / Authority / ExecutionBinding / Attempt / Receipt /
Reconciliation / Outcome / Acceptance / Provenance / Profile`.

## External standards

- JSON Schema 2020-12 for structural contracts.
- CloudEvents 1.0 core attribute names for event envelopes.
- OpenAPI/MCP/A2A as interface/provider protocols, not Morn business truth.
- OCI/Sigstore/SLSA for capability distribution/provenance where configured.
