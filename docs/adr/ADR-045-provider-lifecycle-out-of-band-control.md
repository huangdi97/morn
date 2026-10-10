# ADR-045 — Provider lifecycle control must be out-of-band from blocking execution

- **Status:** Accepted
- **Date:** 2026-10-10
- **Scope:** HarnessProvider / DeepSeek Harness ACP / provider lifecycle / cancellation

## Context

Morn's current `HarnessProvider` seam intentionally presents a small synchronous
execution contract. In particular, `send(&mut self, ...)` owns mutable provider
state until a turn settles. This is suitable for deterministic providers and the
official DSH SDK wire, whose public protocol has no mid-turn cancel/session-close
method.

DeepSeek Harness also exposes an automation-only ACP transport with
`session/cancel`, `session/close`, `session/resume`, listing and permission
requests. Those wire capabilities do **not** automatically mean the existing
Morn provider trait can safely advertise `interrupt=true`: a caller holding the
same provider mutex in a blocking `send` cannot concurrently invoke
`interrupt(&mut self, ...)`.

Advertising cancellation that cannot run concurrently would be a false
capability and could cause operators to believe an external execution was
stopped when it was not.

## Decision

1. **Execution and lifecycle control are separate concurrency domains.**
   Blocking turn execution may use the existing HarnessProvider seam. A provider
   may advertise live interruption/resume/close only when the implementation
   owns an independent control handle that can act without acquiring the
   execution call's exclusive mutable lock.

2. **ACP is implemented first as a lifecycle wire client, not silently promoted
   to HarnessProvider semantics.** The Morn ACP client validates official
   protocol identity, supports new/list/resume/close/cancel/prompt at the wire
   layer, and exercises those calls against a subprocess protocol fixture.

3. **Permission is never Authority.** ACP `session/request_permission` requests
   receive a fail-closed one-shot rejection/cancellation from the generic
   lifecycle client. A future governed ACP Provider must translate an eligible
   request through Morn Capability/Profile/Authority/ExternalAction policy
   before any allow response can exist.

4. **ACP does not widen the effect ceiling.** Production ACP launches reuse the
   same final Morn DSH E0 ToolRuntime deny overlay, read-only permission mode,
   scrubbed child environment and one-shot DSH_HOME as the SDK path. Observed
   ACP tool lifecycle is treated as a policy violation, not as successful work.

5. **Cancel is not rollback.** Cancelling agent activity never proves that an
   already dispatched E1/E2/E3 world effect was reverted. Any ambiguous
   external effect remains `OUTCOME_UNKNOWN` until source-of-truth
   reconciliation.

6. **Resume is exact-history continuation.** A future Work-facing ACP resume
   must revalidate Work id/generation, ExecutionBinding, provider/runtime
   identity, execution environment and scope before the session is allowed to
   continue. Provider replacement creates a new binding; it never edits the
   historical session's binding.

## Consequences

- The ACP wire can mature and be tested without lying about current provider
  feature negotiation.
- `HarnessProviderFeatures.interrupt/resume/session_close` stay false for the
  current DSH SDK provider.
- A future provider-control API should use a clonable/thread-safe control handle
  or command channel, rather than adding more synchronous `&mut self` methods.
- Work/Outcome/Acceptance truth remains independent of both SDK and ACP session
  lifecycle.
- Live ACP runtime evidence remains a deployment gate even when local protocol
  fixture tests pass.

## Rejected alternatives

### Mark DSH as interruptible because ACP has session/cancel

Rejected. Wire support without a concurrently reachable Morn control path is
not an operational capability.

### Kill the child process for every interrupt

Rejected as the semantic definition of cancellation. Process reaping is a
containment fallback; it does not establish per-session persistence semantics
and cannot prove real-world effect rollback.

### Auto-allow ACP permission requests in E0 profiles

Rejected. Tool names and future plugin behavior are not a sufficient Authority
decision. The generic bridge fails closed.

## Verification

Repository-local verification must cover:

- ACP protocol/agent identity validation;
- session new/list/resume/close and cancel wire framing;
- permission rejection;
- visible assistant message projection without private thought projection;
- tool-activity fail-closed behavior;
- bounded wire/process teardown;
- production launch policy isolation.

These tests prove the Morn-side transport only. They do not establish
authenticated LIVE_DSH, production writes, customer Outcome, Acceptance or G12.
