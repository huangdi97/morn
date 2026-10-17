# ADR-013 — Workload identity and credentials are runtime trust artifacts

Status: Accepted for v11.5 convergence.

## Context

Morn already has canonical business identities (Principal/Actor/Workspace) and
also needs short-lived machine/workload identity plus scoped credentials for
providers, connectors and execution environments.

These are different concerns. A harness must not receive a durable production
secret merely because it represents an Actor, and a workload identity must not
replace the human/business Principal accountable for the Work.

## Decision

1. Canonical business identity remains in Morn Identity/Principal semantics.
2. Runtime workloads use a separate `WorkloadIdentityProvider`.
3. The reference shape is SPIFFE-compatible (`spiffe://trust-domain/path`)
   but Morn does not require SPIFFE as its semantic identity system.
4. Workload identity is short-lived and bound to workload + Work + optional
   site + attestation evidence.
5. Secret material is never embedded in WorkloadIdentity records.
6. Credentials are issued as opaque handles, scoped by Work/site/provider/
   resource/action/scopes/TTL, and resolved only at the enforcement/execution
   boundary.
7. Harness prompts/events receive references only when necessary; credential
   values do not become agent memory.
8. Revocation blocks subsequent external effects but does not rewrite earlier
   attempts or receipts.

## Consequences

- SPIFFE/SPIRE, cloud workload identity, mTLS or customer IAM can implement the
  provider later.
- Business accountability stays stable while runtime credentials rotate.
- A compromised harness cannot infer a standing production credential from the
  canonical Work record.
