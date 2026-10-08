# ADR-031 — Durable workflow engines are execution providers, not Work truth

Status: Accepted for v11.5 convergence.

## Context

Morn historically contains a `DurableRuntime` / `WorkflowRun` model with waits,
signals, retry, compensation and checkpoint/recovery. v11.5 additionally has
a desired/observed `WorkResource` plus ExecutionBinding, Attempt, Receipt,
Outcome and Acceptance semantics.

If both models are allowed to become canonical business state, Morn acquires
two competing truths. The same problem would recur when integrating Temporal,
Dapr or another durable workflow engine.

## Decision

1. Canonical business truth remains `WorkResource` and its v11.5 semantic
   records.
2. `WorkflowRun` and external durable-workflow cursors are executor/runtime
   state only.
3. A workflow execution is attached through an explicit
   `DurableWorkflowBinding` that references the exact Morn ExecutionBinding
   and Work generation.
4. Workflow status is normalized into `DurableWorkflowEvidence`.
5. Workflow completion never implies source-grounded Outcome or independent
   Acceptance.
6. Workflow-provider migration creates a new binding; an active run/binding is
   never silently rebound.
7. The legacy Morn `DurableRuntime` is retained as a compatible provider
   implementation, not a second canonical Work model.
8. Temporal/Dapr-class engines may implement the same provider boundary later
   without changing Work identity or acceptance semantics.

## Consequences

- Existing durable-runtime code remains reusable instead of being rewritten.
- A completed Temporal/Dapr/Morn workflow can leave Work in a waiting state
  until authoritative Outcome evidence and Acceptance arrive.
- Harness state, workflow state and Work state remain distinct.
- Process restart/checkpoint behavior belongs to the workflow provider while
  business reconciliation remains in the Morn control plane.
