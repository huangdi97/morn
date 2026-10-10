# ADR-023 — Execution manifests pin interpretation, not business truth

Status: Accepted for v11.5 convergence.

## Context

Morn intentionally allows implementation replacement: Cordis can evolve,
HarnessProviders can change, execution environments can be reselected and
capabilities can receive new releases. That flexibility creates a reproducibility
problem unless each concrete execution scope records the exact semantic/runtime
interpretation that was used.

Recording only a Work id is insufficient. Recording only an Agent/session id is
also insufficient because Work may outlive and replace that executor.

## Decision

A concrete governed execution binding has an immutable-by-reference
`ExecutionManifest` containing at least:

- Morn protocol version;
- Work id and desired-spec generation;
- exact Profile reference;
- site reference;
- source SolutionPackage reference when present;
- ExecutionBinding id;
- capability manifest reference;
- provider id/version and optional content digest;
- reference composition runtime id/version/digest;
- runtime/environment reference when known;
- Authority decision reference when required.

The manifest is generated only when the ExecutionBinding matches the current
Work generation/profile/site.

The manifest is persisted as historical provenance. It does not replace Work,
Attempt, Receipt, Outcome or Acceptance.

## Consequences

1. Provider/runtime upgrades create a new binding/manifest rather than silently
   changing the interpretation of an existing Attempt.
2. Historical replay can answer “which exact protocol/profile/provider/runtime
   produced this evidence?”
3. Morn can replace Cordis in the future without losing the interpretation of
   earlier runs.
4. The composition runtime remains implementation metadata, not canonical
   business truth.
5. Real runtime/package digests should be added whenever a provider can supply
   them; fixture references are engineering evidence only.
