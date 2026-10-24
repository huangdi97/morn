# ADR-017 — Digital Employee is a role interface, not an agent identity

Status: Accepted for v11.5 convergence.

## Context

Morn needs a product concept that organizations can understand ("production
exception coordinator", "quality investigator", "research assistant") without
forcing every stable role to map 1:1 to one LLM/agent runtime.

The repository already has canonical organization semantics:
`RoleSlot`, `MemberBinding`, `ResponsibilityBinding`, Actor/Worker member
types, and dynamic `Workcell`. Creating another DigitalEmployee record would
duplicate those truths.

## Decision

1. "Digital Employee" is a product/organization projection over an existing
   `RoleSlot + MemberBinding`.
2. The role owns stable responsibilities, required capabilities and an
   authority ceiling. It does not own a permanent harness session.
3. A digital role may be filled by Actor, DeterministicWorker or
   ExternalService member types. Humans remain humans; devices remain devices.
4. A Workcell is assembled for concrete Work from the minimum-sufficient
   executor mix. It may use zero, one or multiple agents.
5. One digital role may use different harness/model/solver/tool providers across
   Works, subject to qualification, profile and authority gates.
6. A harness crash, model replacement or provider migration does not delete the
   role or the Work.
7. Digital Employee UI/status is a projection and must not become a second
   canonical state machine beside Work/Organization/ExecutionBinding.

## Consequences

- Product language can use Digital Employee where useful without making Morn
  Agent-first.
- Existing organization contracts remain authoritative and no seventh parallel
  record family is introduced.
- Role continuity is independent from executor/runtime continuity.
- Work remains the coordination unit; Capability remains the composition unit.
