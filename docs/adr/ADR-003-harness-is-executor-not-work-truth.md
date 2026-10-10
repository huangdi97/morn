# ADR-003 — Harness State Is Execution State, Not Work Truth

Status: Accepted for v11.5.

## Decision

DeepSeek Harness, Pi and future agent frameworks implement a HarnessProvider
boundary. Session, turn, step, tool-call and model-output state are execution
evidence only. Canonical Work state remains in Morn.

## Consequences

- A harness crash does not erase or complete Work.
- Harness events may inform Conditions/evidence but cannot directly create an
  accepted business outcome.
- Provider feature support is negotiated; unsupported cancel/resume semantics
  are never fabricated.
