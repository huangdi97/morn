# ADR-044 — External protocol Tasks are executor state, not Morn Work

Status: Accepted for v11.5 convergence.

## Context

Two interoperability protocols now expose durable task-like runtime objects:

- A2A 1.0 uses Tasks/Artifacts to coordinate independent agent systems.
- MCP 2026-07-28 moves long-running execution into the official
  `io.modelcontextprotocol/tasks` extension. MCP Tasks are durable state
  machines used for polling/deferred tool results.

This is valuable execution infrastructure, but it creates a naming trap:
"durable Task" can sound equivalent to a Morn Work. They are not equivalent.

Morn Work represents the business objective, constraints, profile, acceptance,
accountability and desired/observed state across executor replacement. An
A2A/MCP Task represents the state of one external protocol execution.

## Decision

1. A2A Task and MCP Task are **executor runtime evidence**.
2. External task ids never replace Morn Work ids.
3. Every governed external task is associated with an explicit
   `ExecutionBinding`/interop binding.
4. `completed` means the external protocol execution completed. It does not
   create a Morn ObservedOutcome or AcceptanceDecision.
5. `failed`/`cancelled` do not automatically mean the Work failed/cancelled;
   the controller decides whether to rebind, compensate, escalate or terminate.
6. If an external Task has durable polling/resume semantics, Morn persists the
   task handle as provider state/evidence sufficient to resume observation
   after controller restart.
7. Protocol TTL/deletion of the external Task must not delete Morn Work history.
8. A2A protocol version is negotiated at Major.Minor granularity (latest released
   line at this decision is 1.0); patch releases are not semantic protocol
   versions.
9. MCP provider adapters are revision-aware. The current stable MCP revision
   is represented as an adapter/version binding rather than baked into Morn
   semantic records.

## Consequences

```text
Morn Work
   │
   └── ExecutionBinding
          ├── A2A Task
          ├── MCP Task
          ├── DSH session
          ├── Pi session
          └── Workflow run
```

Any of the executor objects may terminate or be replaced while the same Work
continues. This preserves Work-first semantics while allowing Morn to reuse
modern durable task infrastructure instead of rebuilding it.
