# ADR-026 — Runtime signals are not durable business facts

Status: Accepted for v11.5 convergence.

## Context

Cordis and modern harnesses expose rich event taxonomies for plugin, Fiber,
session, model, tool and UI lifecycle. DeepSeek Harness additionally separates
Cordis framework events from its durable session event log.

Morn must preserve the same distinction at the Work-control layer. A provider
reload, Fiber disposal, harness session completion or UI projection update can
be operationally important without becoming canonical Work/Outcome truth.

CloudEvents solves the transport envelope problem but does not determine the
business meaning or durability of an event.

## Decision

Morn classifies events into semantic roles:

- `RuntimeSignal` — provider/plugin/session lifecycle; ephemeral/non-canonical;
- `ControlIntent` — desired-state/control request that must survive restart;
- `DomainFact` — durable fact emitted by the Morn semantic/control plane;
- `ExternalObservation` — durable source-grounded external observation/receipt;
- `ProjectionNotification` — best-effort derived UI/projection notification.

Rules:

1. RuntimeSignal and ProjectionNotification cannot become business truth merely
   because they are delivered reliably.
2. ControlIntent, DomainFact and ExternalObservation require durable handling.
3. ExternalObservation must identify its source-of-truth.
4. Durable semantic events must identify the schema/contract used to interpret
   their payload.
5. CloudEvents remains the wire envelope; Morn adds semantic-role extensions.
6. The durable outbox helper refuses runtime-only event classes.
7. Cordis/Cross-harness event buses remain implementation detail and cannot
   replace Work/Attempt/Receipt/Outcome records.

## Consequences

- Morn can consume high-volume runtime events without polluting the business
  history ledger.
- Provider hot reload remains observable while preserving active binding truth.
- Event transport reliability and business-fact authority remain separate.
