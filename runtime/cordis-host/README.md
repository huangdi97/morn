# Morn Reference Cordis Host

This directory is the reference **Composition Plane** for Morn v11.5.

It uses exact-pinned `@deepseek-ai/cordis@4.0.4` for node-local service slots,
dependency/lifecycle semantics and provider replacement. It does **not** own
durable Work state, external-action truth, receipts, outcomes or acceptance.

## Boundary

Cordis answers which service slots exist, which provider implements a slot, and
how local provider/plugin lifecycle is managed.

The Morn control plane answers what Work is desired/observed, which generation
an ExecutionBinding is pinned to, whether authority was satisfied, whether a
real external action committed, whether reconciliation is required, and whether
an outcome was observed and accepted.

Provider replacement affects future bindings. An active ActionAttempt remains
pinned to its recorded ExecutionBinding until an explicit migration/new attempt
is created.

DeepSeek Harness is intentionally integrated out-of-process through its public
SDK/ACP boundary, so Morn does not share DSH's internal Cordis tree.
