# Morn v11.5 Threat Model

Status: implementation security baseline for the v11.5 convergence branch.

This document describes engineering boundaries and failure handling. It does not
claim a production security audit or a customer-site certification.

## Security objective

Morn coordinates real work across humans, models, agents, rules, solvers,
services, connectors and machines. The primary security objective is therefore
not "make the model safe"; it is:

> an untrusted or mistaken executor must not gain more authority than the
> specific Work/Binding/Profile permits, and uncertain external effects must not
> be silently converted into success or retried blindly.

## Trust boundaries

1. **Semantic/control plane** — Work, Binding, Attempt, Receipt, Outcome,
   Acceptance and Profile are canonical Morn records.
2. **Composition runtime** — Cordis composes local software services. Cordis
   lifecycle is not a business authorization boundary.
3. **Harness/model runtime** — DSH, Pi and other harnesses are replaceable and
   potentially compromised execution providers.
4. **Execution environment** — process/container/microVM/VM/remote/physical
   providers enforce runtime isolation guarantees.
5. **Action/connector boundary** — external side effects cross an enforcement
   point with an explicit authority decision and idempotency/business key.
6. **External source of truth** — ERP/MES/CMMS/SCADA/etc. can disagree with
   harness claims; reconciliation must prefer authoritative observation.
7. **Capability supply chain** — compiler output, qualification, release,
   signing and site admission are separate stages.
8. **Human approval surface** — UI presentation is untrusted input to the human;
   the signed/recorded decision must bind the exact action/binding/context.

## Threats and required controls

| Threat | Example | Required control |
| --- | --- | --- |
| Prompt injection | document tells agent to ignore policy | prompts cannot alter Authority/Profile/Acceptance; tool calls still pass enforcement |
| Tool injection | malicious MCP/tool schema requests dangerous action | capability qualification, allowlisted interfaces, action gateway |
| Malicious capability package | package exfiltrates secrets | content digest, provenance/signature refs, site admission, isolated environment |
| Provider impersonation | fake DSH/Pi endpoint | provider identity/version/digest pinned in ExecutionBinding |
| Credential leakage | token appears in prompt/log | opaque credential handle; resolve secret only at execution boundary; redaction tests |
| Credential over-scope | read task receives write token | credential request scoped to Work/site/resource/action/scopes/TTL |
| Stale authority | approval used after expiry | time-bounded AuthorityRequest; fail closed |
| Revoked authority | operator revokes delegation mid-work | revoked flag/provider decision must block subsequent external action |
| Binding substitution | provider swapped during active attempt | active Attempt pins ExecutionBinding; replacement creates new binding |
| Qualification spoofing | generated skill labels itself certified | compiler emits Declared only; independent QualificationRecord required |
| Site-admission spoofing | capability qualified elsewhere used in plant A | SiteAdmission requires matching site + profile conformance |
| Provenance tampering | package contents changed after evaluation | content digest + package provenance/signature references |
| External-source conflict | model says order exists but CMMS says absent | source-grounded ObservedOutcome; reconcile authoritative system |
| Timeout after commit | POST succeeded remotely but client times out | OUTCOME_UNKNOWN -> RECONCILING; no blind retry |
| Duplicate/replayed action | repeated create-order | business/idempotency key and attempt state machine |
| Compromised harness | agent emits fake "completed" | harness event never becomes Outcome/Acceptance by itself |
| Sandbox escape | local sandbox fails to contain process | profile minimum isolation + provider enforcement report; fail closed |
| Partial sandbox | filesystem sandbox has no network isolation | capabilities declare network/isolation requirements separately |
| Approval confusion | UI shows benign text for different action | approval must bind action/resource/binding/parameters/profile digest |
| Cross-workspace leakage | capability reads another workspace | workspace scope at store/service/connector boundary |
| Cross-site misuse | plant-A admission used for plant-B | site-scoped admission and credentials |
| Event duplication | bus redelivers event | event id/dedupe + idempotent controller reconciliation |
| Event reorder | delayed observation arrives after newer state | sequence/source time + bitemporal history; do not silently overwrite |
| Clock skew | expiry/observation ordering distorted | timestamps are evidence, not sole causal proof; source sequence where available |
| History rewrite | old evidence edited after decision | append/supersede/retract/redact with explicit relation and actor |
| Supply-chain rollback | vulnerable older package reintroduced | release/admission lifecycle + digest pin + explicit migration |
| Profile downgrade | runtime removes required guarantee | conformance gate on exact profile/version before new binding |
| Budget bypass | provider cost exceeds Work limit | resolver hard budget gate + attempt accounting |
| Unbounded delegation | subagents recursively create more authority | delegation depth/budget/scope ceilings are explicit Authority fields |
| Human-role spoofing | model claims to be approver | identity and acting-role decision come from authority/identity provider, not prompt text |

## DeepSeek Harness boundary

The current DSH SDK wire is newline-delimited JSON-RPC over stdio. Its current
request methods are `initialize`, `session/prompt`, and `shutdown`; current
server notifications include `session.event`, `session.status`,
`subagent.started`, and `subagent.finished`.

The current SDK protocol does not expose a turn-cancel or session-close method.
Morn therefore records optional harness lifecycle features explicitly and must
not claim cancellation/resume semantics merely because the generic
`HarnessProvider` trait has such operations. Closing the whole runtime process
is not equivalent to safely cancelling one Morn Work/Attempt.

DSH session events are execution evidence, not Morn Work truth.

## Sandbox boundary

A same-world process sandbox is not automatically a network/process isolation
boundary. Morn's `ExecutionEnvironmentSpec` therefore separates isolation,
runtime, network allowlist, writable paths, secret handles, resources, timeout
and side-effect policy.

A profile requiring microVM/VM/network isolation cannot be satisfied by a
provider that only promises filesystem confinement.

## Capability supply-chain boundary

The following transitions prove different facts and cannot be collapsed:

```text
Artifact
 -> Compiler
 -> Declared
 -> Observed/Evaluated
 -> Qualified
 -> Release/package digest
 -> Profile conformance
 -> Site admitted
```

A generated capability is never admitted because generation succeeded.
Qualification records carry test-suite references, environment digest, expected
properties, failure modes, evaluator identity and validity window. Admission
fails closed when qualification expires or profile conformance fails.

## Real-world effect boundary

A real external action must bind:

- Work id/generation;
- ExecutionBinding id;
- capability/provider version or digest;
- authority decision;
- action/resource/parameter envelope;
- business/idempotency key;
- site/profile;
- execution environment/credential handles where applicable.

Outcome states distinguish dispatch, acknowledgement, external commit,
observation and verification. `OUTCOME_UNKNOWN` is a normal recoverable state.

## Historical integrity and privacy

Morn does not require permanent retention of every payload. It requires
non-silent historical evolution.

- Current projections may change.
- A correction can supersede a prior fact.
- A false fact can be retracted.
- A privacy/legal request can redact payload while retaining an auditable
  tombstone when permitted.
- `valid_time` and `recorded_at` support "what was true" vs "what Morn knew
  when" analysis.

Retention/deletion policy remains Profile/deployment specific.

## Security gates for Factory

`morn.factory.readonly@1.0.0` forbids `ProductionWrite`. Fixture code may
exercise an external-action/reconciliation state machine, but this does not
authorize or claim production write capability.

Before a future write-enabled Factory profile can exist, at minimum:

1. real site identity/IAM integration;
2. site-specific source-of-truth mapping;
3. capability qualification and site admission;
4. enforced idempotency/reconciliation;
5. isolation/network/secret guarantees proven at the site;
6. explicit human/customer authority envelope;
7. failure/rollback/compensation plan by effect class;
8. audit/evidence retention policy;
9. shadow evidence and acceptance criteria;
10. production owner approval for the exact profile version.

## Required failure behavior

A security-relevant ambiguity must resolve to one of:

- block;
- request approval;
- reconcile;
- suspend;
- compensate;
- escalate;
- revoke;
- fail closed.

It must never silently downgrade a requirement, fabricate evidence, bypass a
profile gate, or reinterpret a harness/model statement as authoritative
business truth.


## Workload identity boundary

Canonical Principal/Actor identity and runtime workload identity are separate.
A workload identity is short-lived, bound to a Work/workload/site and may be
represented by a SPIFFE-compatible URI. It contains no secret material.
Credential values remain behind opaque handles and are resolved only at the
execution/enforcement boundary.

Remote/physical execution labels are not treated as stronger isolation than a
microVM/full VM. A Factory isolation requirement must be satisfied by explicit
provider guarantees rather than enum ordering.
