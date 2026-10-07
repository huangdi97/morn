# ADR-008 — Prefer Existing Interoperability Standards

Status: Accepted for v11.5.

## Decision

Morn will not invent proprietary equivalents when a suitable standard exists.

Preferred boundaries include MCP/OpenAPI for tools/services, A2A for agent
peers, CloudEvents-compatible event envelopes, OpenTelemetry semantic
attributes, OCI/ORAS package distribution, Sigstore/SLSA signature/provenance,
and OPA/Cedar/customer IAM for policy decisions.

## Consequences

Morn concentrates innovation on Work semantics, binding, authority,
reconciliation, outcome and acceptance rather than protocol reinvention.
