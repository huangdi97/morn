# ADR-016 — Morn has replaceable implementations but non-bypassable semantic laws

Status: Accepted for v11.5 convergence.

## Context

The architecture should be highly composable: Cordis or another composition runtime,
DSH/Pi/other harnesses, different workflow engines, sandboxes, policy engines,
connectors, models, solvers and domain packs should be replaceable.

However, making every rule a plugin would let a plugin redefine the meaning of
Work, authority, outcome or history and would destroy interoperability and auditability.

## Decision

Morn does not freeze one implementation. It freezes a small versioned semantic constitution:

1. Work identity and generation are explicit.
2. Capability declaration, qualification, release and site admission are distinct facts.
3. Authority decision precedes governed side effects and is separate from execution.
4. A running Attempt is pinned to an ExecutionBinding.
5. Dispatch/ack/commit/observation/verification are distinct external-effect states.
6. Unknown external outcome is explicit and reconciled before redispatch.
7. Harness/runtime completion is execution evidence, not business Outcome.
8. Outcome is source-grounded and Acceptance is independently evaluated.
9. Historical correction is explicit; past interpretation is not silently overwritten.
10. Domain Profiles specify guarantees, not provider names.
11. Provider/runtime migration creates a new binding/decision rather than editing history.
12. Fixtures and simulations cannot be promoted into real customer/production evidence.

These laws are versioned protocol semantics. Their concrete stores, controllers,
composition runtime, policy engine, sandbox, workflow engine and user interfaces remain replaceable.

## What can be plugged/replaced

- Cordis composition host;
- DeepSeek Harness, Pi, AgentScope/OpenAI-style agent runtimes;
- deterministic programs, rules, solvers, humans, devices;
- Temporal/Dapr-class workflow providers;
- process/container/microVM/VM/remote execution providers;
- OPA/Cedar/customer IAM authority providers;
- SPIFFE/cloud workload identity providers;
- MCP/OpenAPI/A2A adapters;
- OCI registry/signature/provenance backends;
- domain packs, UI extensions and SolutionPackages.

## What a plugin cannot do

- silently change the semantic meaning of a published protocol version;
- mutate an active binding in place;
- grant itself authority;
- turn its own success message into accepted business outcome;
- rewrite historical evidence without an explicit supersede/retract/redact relation;
- lower a selected Profile guarantee without an explicit new profile/binding decision.

## Consequence

The answer to "what is immutable?" is not Cordis, Factory Profile or a Rust crate.
The stable part is the versioned semantic contract plus non-destructive history.
Everything else should be replaceable behind conformance-tested boundaries.
