# ADR-040 — Local control-plane browser origins are not trusted by default

Status: Accepted for v11.5 convergence.

## Context

The reference Morn server binds to loopback, but loopback alone is not a
browser security boundary. A malicious remote web page can attempt cross-origin
requests to `127.0.0.1`/localhost. With permissive CORS and unauthenticated
mutation endpoints, browser-originated CSRF can mutate local control-plane
state even though the service is not network-exposed.

## Decision

1. The reference server remains loopback-bound by default.
2. Permissive CORS is removed.
3. Browser origins are explicitly allowlisted for the reference UI/Tauri shell.
4. Mutating requests carrying an `Origin` header are rejected unless the
   origin is an approved Morn/Tauri origin.
5. Origin-less requests are allowed for local CLI/native/provider traffic; this
   is not treated as an enterprise authentication mechanism.
6. GET/HEAD remain readable under normal browser CORS rules; OPTIONS is allowed
   for preflight.
7. This local-origin guard does not replace future user/workload
   authentication, authorization or mTLS for a remotely deployed control plane.

## Consequences

- A random internet page cannot use the browser as a confused deputy to POST
  to the default localhost Morn server.
- Existing curl/CLI smoke tests remain possible without inventing credentials.
- Real enterprise/network deployment remains external-blocked until proper
  identity/authentication/TLS policy is configured.
