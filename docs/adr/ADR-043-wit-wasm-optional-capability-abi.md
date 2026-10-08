# ADR-043 — WIT/Wasm Component Model is an optional local capability ABI

Status: Accepted as a deferred provider option for v11.5; not required for the
reference runtime.

## Context

Cordis is the reference node-local composition runtime and is TypeScript/Node
centric. Morn capabilities can be implemented in Rust, Python, Go, Java, C/C++,
R, vendor binaries, remote services or physical systems. The Morn Protocol wire
surface therefore cannot depend on an in-process TypeScript interface.

The WebAssembly Component Model uses WIT interfaces to define imports/exports
independently of implementation language. WASI 0.3 adds native async,
streams and futures, improving long-running and streaming component
composition. This makes the Component Model a plausible future ABI for small,
portable, least-capability local providers.

## Decision

1. WIT/Wasm is **not** a new Morn core dependency and does not replace Cordis.
2. Morn may add a future `WasmComponentProvider` for capabilities that benefit
   from portable typed local execution.
3. A WIT world/interface maps to a Capability interface contract; missing
   imports are treated as unavailable capabilities, not as implicit ambient
   authority.
4. Host-provided filesystem/network/secret/time/random interfaces remain
   governed by ExecutionEnvironment and Authority/Profile constraints.
5. A Wasm component cannot own canonical Work state, Authority, Outcome or
   Acceptance.
6. WASI/WIT version pins are part of the provider/runtime binding and
   qualification evidence.
7. Do not force remote APIs, human roles, agents or physical executors through
   Wasm merely for uniformity.

## Consequences

- Morn retains cross-language portability without inventing a proprietary local
  plugin ABI.
- Cordis continues to compose node-local services while a Cordis service may
  proxy a Wasm capability.
- Future edge/industrial deployments can evaluate Wasm as one execution class
  without redesigning the semantic protocol.
