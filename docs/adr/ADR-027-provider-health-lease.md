# ADR-027 — Provider health is leased evidence, not a permanent property

Status: Accepted for v11.5 convergence.

## Context

Composable runtimes make providers dynamic: a harness process can crash, a
remote agent can disappear, credentials can expire, a solver service can become
degraded, or a plugin dependency can unload.

A capability manifest naming a provider is not sufficient proof that the
provider is currently selectable. Likewise, a health probe from hours ago
should not make an executor eligible forever.

## Decision

1. ProviderRegistry health is a mutable projection backed by append-only
   ProviderObservation history.
2. Healthy providers may carry `health_valid_until`; after that time the
   health claim is stale and the provider is not selectable.
3. Provider selection uses `selectable_at(now)`, not a timeless status flag.
4. The control plane applies a ProviderGate before semantic capability
   resolution when strict runtime evidence is required.
5. Enterprise/Factory compositions should require providers to be registered
   and currently selectable.
6. Lite/local compositions may deliberately permit unmanaged local providers.
7. Provider disappearance never rewrites an active ExecutionBinding or Attempt.
   It only affects future binding/rebinding decisions.
8. Provider recovery produces a new observation; it does not erase outage
   history.

## Consequences

- Dynamic Cordis/DSH/Pi/provider lifecycle changes are reflected without
  conflating runtime health with Work identity.
- Capability admission and provider liveness remain distinct gates.
- Stale health evidence fails closed for strict profiles.
