# ADR-009 — Execution Isolation and Credentials Are Provider Boundaries

Status: Accepted for v11.5.

## Decision

Sandbox is not a single implementation. Capability requirements select an
ExecutionEnvironmentProvider across process/container/microVM/VM/remote/physical
classes.

Secrets are represented to Morn/harnesses as opaque, scoped credential handles.
Actual credential values are resolved only at the enforcement/execution
boundary.

## Consequences

A DSH/Pi local sandbox cannot satisfy stronger isolation by declaration alone.
Credentials do not become prompt or agent-memory data.
