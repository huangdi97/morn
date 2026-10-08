# ADR-029 — Pi integrates through its public JSONL RPC subprocess boundary

Status: Accepted for v11.5 convergence.

## Context

Current Pi exposes a language-neutral long-lived RPC mode:

```text
pi --mode rpc --no-session
```

Commands and responses are strict LF-delimited JSON records on stdin/stdout.
A successful `prompt` response means only accepted/queued/handled; execution
may continue through retries, compaction or follow-up work. `agent_settled`
is the executor-level idle boundary. Pi also exposes `get_state` and
`abort`.

Morn is written primarily in Rust and must not bind its semantic model to Pi's
in-process TypeScript SDK or npm package identity.

## Decision

1. The reference Pi integration uses the public subprocess RPC protocol.
2. Default launch is `pi --mode rpc --no-session`; provider/model/cwd are
   runtime configuration.
3. Framing is strict JSONL over stdout/stdin; stderr is diagnostics only.
4. Commands use correlation ids; session events are consumed independently.
5. `prompt success` is not executor completion.
6. `agent_end` is not necessarily final; `agent_settled` is the boundary
   used when Morn needs Pi's session-level run to become idle.
7. Neither prompt success nor agent_settled equals Morn Outcome or Acceptance.
8. Pi process failure affects runtime/provider health and future bindings, not
   canonical Work identity.
9. Morn does not freeze an npm package name as protocol identity; the CLI/wire
   contract is the provider boundary.

## Consequences

- Pi remains replaceable with DSH or another HarnessProvider.
- Morn can integrate Pi from Rust without embedding a Node runtime into the
  semantic/control plane.
- Real smoke still requires an installed Pi binary, configured model/provider
  and credentials; absence remains an external/runtime blocker rather than a
  fixture PASS.
