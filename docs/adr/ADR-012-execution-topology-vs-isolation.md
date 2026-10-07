# ADR-012 — Execution topology is not an isolation ordering

Status: Accepted for v11.5 convergence.

## Context

Agent runtimes increasingly expose heterogeneous execution backends: local
processes, containers, microVMs, full VMs, remote sandbox services and physical
executors. DSec-style elastic sandbox infrastructure and AgentScope Runtime both
reinforce that these are different execution environments with different
properties, not one scalar ladder.

A remote executor is not automatically "more isolated" than a microVM. A
physical executor is not automatically "more isolated" than a full VM. Treating
those labels as an ordinal security rank can accidentally satisfy a strict
Factory profile with an environment whose actual guarantees are unknown.

## Decision

1. Process/container/microVM/full-VM may be treated as a local containment
   hierarchy only where the provider explicitly supports those classes.
2. Remote and physical are topology/executor classes, not higher ranks.
3. A remote requirement is satisfied only by an explicitly remote provider;
   a physical requirement only by an explicitly physical provider.
4. A profile requiring container/microVM/VM containment cannot be satisfied by
   a generic remote/physical label alone.
5. ExecutionEnvironmentProvider advertises supported classes explicitly instead
   of a single `maximum_isolation`.
6. Future richer guarantees (network isolation, kernel boundary, attestation,
   filesystem policy, secret handling) should be evaluated as independent
   properties rather than compressed into one enum.
7. Capability resolution and Profile conformance must use the same semantics.

## Consequences

- Resolver/profile admission fail closed when an execution topology is not
  sufficient evidence of the requested containment.
- DSH/Pi/AgentScope/DSec-like backends can be added as providers without
  pretending their security properties are interchangeable.
- Site qualification must attach evidence for the concrete provider/backend,
  not merely a label such as "remote".
