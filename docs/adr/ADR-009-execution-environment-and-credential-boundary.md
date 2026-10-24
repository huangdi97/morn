# ADR-009 — Execution Isolation and Credentials Are Provider Boundaries

Status: Accepted for v11.5.

## Decision

Sandbox is not a single implementation. Capability requirements select an
ExecutionEnvironmentProvider across process/container/microVM/VM/remote/physical
classes.

Secrets are represented to Morn/harnesses as opaque, scoped credential handles.
Actual credential values are resolved only at the enforcement/execution
boundary.

A real Harness launch is also pinned to the **identity** of its execution
environment, not only to an isolation enum. The deployment/provider launch
configuration, RuntimeContext, ExecutionBinding and ExecutionManifest must
refer to the same `execution_environment_ref`. A mismatched or missing ref
fails closed even when the caller supplies a sufficient-looking guarantee
vector.

The identity match is necessary but not self-authenticating: the concrete
ExecutionEnvironmentProvider/deployment must still prove that the configured
command actually executes inside that environment. Morn never infers
containment from the presence of Docker, a binary name, a remote endpoint or a
caller-supplied label.

## Consequences

A DSH/Pi local sandbox cannot satisfy stronger isolation by declaration alone.
Provider replacement clears provider-bound runtime/environment/authority state
and requires a freshly resolved environment identity. Credentials do not become
prompt or agent-memory data.


## Tool mediation addendum

For external Harness providers admitted through the E0 execution seam, isolation
alone is insufficient. The attested execution environment must also carry the
provider-neutral `tool-mediation` guarantee: embedded provider tools are
disabled, or every tool invocation is routed through the Morn-governed
Capability / ExternalAction enforcement boundary. Detecting a provider tool
event after it has run is defense-in-depth only and does not satisfy this
guarantee by itself.


## Provider route pinning

A runtime distribution digest identifies executable code, not the route selected
inside that executable. Real Harness bindings therefore also pin a secret-free
`runtime_ref` derived from the configured profile / protocol arguments and
provider-model route. Before every new turn Morn compares the immutable binding
with the currently configured route. Restarting the same signed runtime with a
different model or execution profile requires a new binding; it may not silently
reinterpret existing Work.

Credentials, homes and raw secret values are deliberately excluded from this
route reference. Their authorization remains an execution-environment /
credential-boundary concern.


## Pi RPC concrete tool suppression

For the official Pi CLI path, Morn can enforce part of `tool-mediation` at
process launch rather than relying only on post-run event inspection. The real
RPC default includes `--no-tools --no-mcp`, and real configurations using the
ordinary Pi route arguments fail validation if either flag is removed.

A test/wrapper executable may set `append_route_args = false`; that path is
used by protocol fixtures and still requires the external execution-environment
attestation. It must not be treated as evidence that a production Pi binary was
tool-free.


## DeepSeek SDK concrete tool suppression

The official DSH `sdk` profile inherits the base tool registry and sandbox
configuration. Morn therefore does not accept post-facto `tool/call`
observation as its primary enforcement boundary.

A production DSH Real launch is fixed to exactly `dsh --profile sdk`, followed
by one Morn-owned **last** `--patch` overlay. Arbitrary launcher/application
arguments or another profile fail validation before the subprocess starts. The
overlay:

1. inserts a Morn Cordis plugin whose global monotonic `ctx.tools.guard()`
   rejects every direct ToolRuntime execution, including tools introduced by
   earlier Profile/Home layers that Morn does not know by name;
2. disables the currently shipped model-facing tool producers so forbidden
   tools are also removed from the model-visible surface; and
3. sets the DSH sandbox to `read-only`; process launch independently forces
   `DSH_PERMISSION_MODE=read-only`.

The overlay and guard-plugin bytes are hashed together with SHA-256 and that
policy identity is part of the secret-free provider route reference. Policy
drift therefore requires a new ExecutionBinding instead of silently
reinterpreting existing Work.

Every launch receives a unique policy directory under the isolated `DSH_HOME`;
the overlay and guard are created exclusively and removed only after the owned
subprocess is reaped. The workspace and Harness home are canonicalized and must
remain physically disjoint. A protocol-fixture bypass exists only under Rust
`cfg(test)`; the toggle is private and skipped by serde, so production callers
and serialized configuration cannot disable the policy.

This concrete launch policy is defence in depth, not a replacement for the
provider-neutral `tool-mediation` execution-environment attestation. The
attestation still binds the exact runtime artifact and containment boundary,
while the Morn overlay constrains the pinned DSH composition.
