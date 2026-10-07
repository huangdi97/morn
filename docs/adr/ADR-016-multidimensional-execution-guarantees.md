# ADR-016 — Execution guarantees are multidimensional, not a sandbox brand

Status: Accepted for v11.5 convergence.

## Context

An execution backend can be a process, container, microVM, full VM, remote
service or physical executor. None of those labels alone proves filesystem,
network, secret, resource, statefulness or attestation properties.

DSec demonstrates a unified execution platform spanning multiple sandbox
substrates, while other agent runtimes expose their own sandbox and state
semantics. Morn must select backends by required guarantees rather than by
brand or a single ordinal isolation number.

## Decision

Execution requirements use two independent dimensions:

1. **ExecutionClass** — where/how the workload is hosted:
   no-isolation, process, container, microVM, full-VM, remote, physical.
2. **ExecutionGuarantee** — what properties are proven:
   filesystem read/write policy, network egress policy, process/kernel
   boundary, resource limits, secret indirection, workload identity,
   stateful execution, checkpoint/resume and runtime attestation.

CapabilityManifest declares required execution guarantees. CapabilityRequest
can require guarantees. DomainProfile declares a guarantee floor. An
ExecutionEnvironmentProvider must advertise/prove what it satisfies.

Resolver and profile conformance fail closed when required guarantees are
missing. A stronger-looking topology label cannot substitute for missing
evidence.

## Factory read-only baseline

The first Factory read-only profile currently requires at least:

- container-compatible containment;
- filesystem-write policy;
- network-egress policy;
- secret indirection;
- plus the semantic Work/Authority/Receipt/Reconciliation/Outcome/Acceptance
  guarantees already defined by the profile.

This remains an engineering guarantee profile, not authorization for
production writes.

## Consequences

- DSec-like, AgentScope-like, cloud sandbox, Kubernetes and local execution
  backends can compete behind one provider boundary.
- A remote sandbox does not automatically satisfy a microVM requirement.
- Security review can add guarantees without changing canonical Work semantics.
- Site admission can bind concrete evidence to the exact backend/version.
