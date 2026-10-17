# ADR-033 — Engineering evidence class and value-validation class are orthogonal

Status: Accepted for v11.5 convergence.

## Context

Morn has two distinct questions:

1. What environment/class of evidence exists for a claim?
2. What kind of value evaluation was performed for a Work outcome?

`EvidenceClass` answers the first question (design, fixture, CI, real runtime,
real site, production write). `ValueEvidenceClass` answers the second
(fixture, simulation, shadow, observed operational, customer validated).

Collapsing these axes would allow a `CustomerValidated` label to fabricate
missing real-site evidence, or would incorrectly require production write for
a legitimate read-only customer pilot.

## Decision

1. `EvidenceClass` and `ValueEvidenceClass` remain separate typed vocabularies.
2. Both are categorical rather than maturity inheritance ladders.
3. A customer-value claim requires:
   - `ValueEvidenceClass::CustomerValidated`;
   - an independent AcceptanceDecision reference;
   - explicit value evidence references;
   - an explicit `EvidenceClass::RealSite` claim for the value subject.
4. ProductionWrite evidence is not required merely to prove customer value;
   read-only/shadow customer pilots may validate value.
5. A production-write claim still requires its own explicit ProductionWrite
   evidence class.
6. UI/reports must not present fixture/simulation/shadow metrics as customer
   value.

## Consequences

- Value and deployment maturity cannot silently inflate each other.
- Morn can validate real business value before permitting consequential writes.
- The first Factory wedge remains compatible with read-first customer pilots.
