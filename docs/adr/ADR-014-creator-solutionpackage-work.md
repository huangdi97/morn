# ADR-014 — Creator is a translation layer; SolutionPackage is the reusable blueprint

Status: Accepted for v11.5 convergence.

## Context

Morn should let a user create useful compositions with very little ceremony:
describe a goal, choose a guarantee profile, state acceptance criteria, and make
only the site/capability choices that are actually needed.

At the same time, introducing separate canonical objects named Blueprint,
AgentInstance, DigitalEmployee, FactoryWorker, etc. would recreate the same
business truth in multiple object models.

## Decision

1. **Creator is product UX, not canonical truth.** Its input is a transient
   `CreatorRequest`.
2. Creator translates into the existing
   `ProblemSpec -> WorkGraph -> ProposedSolution` pipeline.
3. After review/approval, the existing `SolutionPackage` is the reusable
   blueprint/package.
4. Instantiating a SolutionPackage creates a canonical `WorkResource`.
5. Runtime harness sessions, A2A tasks, agent instances and Cordis services are
   executor/runtime state attached to an `ExecutionBinding`; none becomes a
   second Work identity.
6. A user can choose three autonomy postures:
   - Assist;
   - Governed;
   - AutonomousWithinPolicy.
   These alter planning/risk posture only; they never bypass Authority,
   Profile, Acceptance, qualification or site admission.
7. Creator requires explicit acceptance criteria before it can draft a
   composition.
8. Guarantee Profile is chosen before execution; Provider names are resolved
   later from required capabilities and guarantees.
9. A Factory read-only profile automatically retains the ProductionWrite
   prohibition and source-of-truth/site readiness gates.

## User-facing mental model

```text
"I need X"
  -> Creator
  -> reviewable solution draft
  -> approved SolutionPackage
  -> instantiate Work
  -> resolve minimum-sufficient Workcell
  -> bind exact providers/runtime/authority
  -> execute
  -> receipt/reconcile
  -> observed outcome
  -> independent acceptance
```

A simple Work may resolve to one deterministic program and no agent. A complex
Work may resolve to humans + rules + solver + one or more harness-backed agents.

## Consequences

- Morn can feel like a lightweight "build your own worker" product without
  becoming an Agent-first framework.
- Domain products (Factory, Research/BioLab, Enterprise, Personal) can ship
  templates and Profiles without forking the core object model.
- Provider ecosystems such as DSH, Pi, AgentScope, OpenAI Agents SDK, MCP and
  A2A remain replaceable execution/interoperability choices.
- Existing SolutionPackage and Work contracts remain authoritative.
