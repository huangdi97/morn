# ADR-041 — Publish a language-neutral Morn protocol surface

Status: Accepted for v11.5 convergence.

## Context

Morn is now defined as a protocol-driven Work control plane. If the semantic
contract exists only as Rust structs, then the implementation language silently
becomes the protocol and third-party runtimes cannot independently implement or
validate Morn semantics.

The architecture already requires replaceable composition runtimes, harnesses,
policy providers, execution environments and external systems. Therefore the
cross-runtime contract needs a language-neutral publication surface.

OpenAPI 3.1 aligns with JSON Schema 2020-12 for data schemas, and Kubernetes-style
control planes demonstrate the value of stable spec/status resources and
observed-generation semantics. Morn can reuse those patterns without adopting
Kubernetes as a runtime dependency.

## Decision

1. Publish `spec/v11.5/protocol.json` as the versioned protocol manifest.
2. Publish `spec/v11.5/morn-protocol.schema.json` using JSON Schema 2020-12.
3. Publish canonical examples for WorkResource, CapabilityRecord,
   ExecutionBinding, ActionAttempt, DomainProfile and EventEnvelope.
4. Reference implementation tests must deserialize canonical examples into the
   real Rust types so structural drift is detected by CI.
5. The schema validates structure. Cross-record behavioral laws remain defined
   by the protocol/ADRs and conformance suite; schema alone is not sufficient
   evidence of Morn compliance.
6. Provider-specific/session-specific objects stay outside the semantic schema
   unless they cross a public provider boundary.
7. A same-version semantic schema mutation is forbidden. Semantic changes use
   explicit protocol/profile version evolution.
8. Cordis/DSH/Pi types are not embedded into the Morn protocol schema.

## Consequences

- A non-Rust runtime can implement the published contract without importing
  Morn crates.
- Rust remains the reference runtime, not the semantic owner.
- Schema and canonical examples become release artifacts and compatibility
  evidence.
- Future OpenAPI/A2A/MCP/xRegistry/WIT projections can reference the same
  canonical Morn schemas rather than defining parallel business truth.
