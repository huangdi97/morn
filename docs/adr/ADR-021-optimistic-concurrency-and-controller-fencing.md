# ADR-021 — Durable Work projections use optimistic concurrency and controller fencing

Status: Accepted for v11.5 convergence.

## Context

The v11.5 control plane is reconciliation-driven. Multiple controller workers
may observe the same Work, provider health event, approval or external receipt.
Without concurrency control, a stale controller can overwrite a newer Condition
or phase even if every individual controller is logically idempotent.

Leader election alone is insufficient: an old controller may continue running
briefly after its lease expires (process pause, network partition, delayed
scheduler). This is the classic split-brain/fencing problem.

## Decision

1. Mutable durable projections carry a store revision independent from Work
   `generation`.
2. `generation` changes when desired Work spec changes.
3. `resource_version` / store revision changes on each successful durable
   projection write.
4. v11.5 controllers use compare-and-swap persistence. A stale
   `resource_version` fails with Conflict instead of last-writer-wins.
5. Controller coordination uses durable leases with monotonic fencing tokens.
6. Lease takeover after expiry increments the fencing token.
7. A stale holder cannot renew after takeover.
8. Future distributed stores/dispatchers must propagate the fencing token to
   any operation where stale-controller effects would be dangerous.
9. External business effects still require ExecutionBinding + Authority +
   ExternalActionPermit. A controller lease never grants business authority.
10. Inbox dedupe, CAS and controller fencing are complementary:
    - inbox: duplicate delivery defense;
    - CAS: stale projection defense;
    - fencing: stale controller/leader defense;
    - Attempt/Reconciliation: external-effect truth.

## Consequences

- Work controllers may run redundantly without silently losing Conditions.
- A provider/runtime outage cannot be "fixed" by letting two controller leaders
  race the same mutable projection.
- Work `generation` is not abused as a persistence revision.
- The SQLite reference adapter implements the minimal contract. A PostgreSQL,
  etcd or cloud-control-plane backend may replace it while preserving the
  semantics.
