# ADR-011 — MCP/A2A are interoperability boundaries, not Morn business truth

Status: Accepted for v11.5 convergence.

## Context

Morn must interoperate with tool/resource protocols and remote agent systems
without allowing those protocols to redefine Morn's Work, Authority, Outcome or
Acceptance semantics.

MCP describes callable tools/resources. A tool description can contain schemas
and annotations, but those declarations do not grant execution authority.

A2A describes independent agent systems, messages, tasks and artifacts. An A2A
Task is meaningful inside the remote agent protocol, but it is still executor
runtime state from Morn's point of view.

## Decision

1. MCP tool/resource metadata maps to capability interface metadata only.
2. Authority is evaluated and enforced separately at the Morn action/connector
   boundary.
3. A2A Task ids never replace Morn Work ids.
4. A2A Task completion never implies Morn Outcome verification or Acceptance.
5. A2A artifacts/messages may become evidence references or candidate artifacts.
6. Any MCP/A2A invocation used in governed Work must be tied to an explicit
   Morn ExecutionBinding.
7. Protocol/runtime upgrades create new provider/interface bindings rather than
   silently changing an active Attempt.

## Consequences

- Morn can adopt new MCP/A2A protocol versions without redefining its business
  semantics.
- Remote agent completion is useful evidence but cannot fabricate customer or
  operational success.
- Security decisions remain independent from self-described tool/agent
  capabilities.
- The integration crate exposes explicit MCP/A2A descriptors and binding types
  while keeping canonical Work records elsewhere.


## Current protocol research note

The interoperability boundary is deliberately version-neutral. Current public
A2A documentation identifies **1.0.0** as the latest released specification;
its Task remains an A2A-managed stateful unit with Messages/Artifacts around it.
That reinforces, rather than removes, the boundary: an A2A Task is a remote
agent protocol object, not a Morn Work identity.

Current MCP HTTP authorization requires resource-bound authorization flows when
authorization is used. The client identifies the target protected resource, and
credential material must not be passed through indiscriminately to another
server. Morn therefore carries an optional resource/audience reference on
credential requests/handles and keeps credential resolution behind the
execution/enforcement boundary.

These protocol details may evolve. Morn pins/records the concrete protocol
version on an interface/provider binding when needed, while the Work/Authority/
Outcome/Acceptance distinction stays in the Morn protocol version.
