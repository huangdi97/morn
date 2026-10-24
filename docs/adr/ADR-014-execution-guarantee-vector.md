# ADR-014 — Execution security is a guarantee vector, not a sandbox label

Status: Accepted for v11.5 convergence.

## Context

Morn can execute through local processes, containers, microVMs, full VMs,
remote sandbox services and physical executors. Contemporary agent runtimes also
separate these concerns: an agent harness may expose a filesystem sandbox while
network egress, kernel isolation, state persistence or workload attestation are
provided by a different layer.

A single label such as `sandboxed`, `remote` or `microVM` is therefore
insufficient evidence for a Factory guarantee profile. In particular, a remote
executor is not automatically stronger than a local microVM, and filesystem
write confinement does not prove network egress policy.

## Decision

Morn defines a provider-neutral `ExecutionGuarantee` vocabulary:

- filesystem read policy;
- filesystem write policy;
- network egress policy;
- process boundary;
- kernel boundary;
- resource limits;
- secret indirection;
- workload identity;
- stateful execution;
- checkpoint/resume;
- runtime attestation.

Capability manifests declare the guarantees they require. Domain Profiles
declare the guarantees a composition must prove. Execution environment
providers advertise concrete guarantees for a selected execution class, and
provisioning fails closed when a required guarantee is absent.

Topology and guarantees remain separate. `Remote` and `Physical` are
executor classes, not higher points on an isolation ranking.

The first Factory read-only profile requires at least:

- filesystem write policy;
- network egress policy;
- secret indirection;
- container-compatible local containment when the reference local environment
  is selected.

These are engineering guarantees. A real site must still supply provider/site
evidence before admission.

## Consequences

A DSH, Pi, AgentScope Runtime, DSec-like service, container engine or future
sandbox can implement the execution-provider boundary without becoming a Morn
semantic dependency.

A provider cannot satisfy Factory merely by naming itself “sandboxed”. Profile
conformance and capability resolution evaluate explicit guarantees.

Adding a new execution backend does not change Work, Authority, Binding,
Attempt, Outcome or Acceptance semantics.


## Requirement versus evidence

Execution requirements and execution evidence are deliberately separate.

A CapabilityManifest says what environment a capability **needs**. A Work or
Domain Profile may impose a stronger floor. Resolution computes the effective
requirement by merging those requirements; it does not reject a capability just
because the capability itself did not redundantly declare every Profile
guarantee.

For example:

```text
capability requires: process + filesystem-read-policy
Factory profile requires: container + network-egress-policy + secret-indirection

effective environment requirement:
container
+ filesystem-read-policy
+ network-egress-policy
+ secret-indirection
```

Only the selected ExecutionEnvironmentProvider can supply evidence that those
effective requirements are actually satisfied. Capability declarations are not
self-attestation.

Incomparable topology requirements fail closed. A remote-only capability cannot
silently satisfy a local container/microVM requirement, and vice versa, without
an explicit provider-level mapping backed by evidence.
