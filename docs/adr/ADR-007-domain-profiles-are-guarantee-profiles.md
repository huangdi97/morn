# ADR-007 — Domain Profiles Specify Guarantees, Not Plugin Lists

Status: Accepted for v11.5.

## Decision

A Morn Profile states semantic and operational guarantees a composition must
meet. It does not name mandatory vendors/providers.

Reference profiles include Lite, Enterprise, Factory Readonly and Research.
The Factory Readonly profile explicitly forbids ProductionWrite.

## Consequences

OPA/Cedar, DSH/Pi, Temporal/Dapr and customer systems may vary while the same
Profile remains testable through conformance evidence.
