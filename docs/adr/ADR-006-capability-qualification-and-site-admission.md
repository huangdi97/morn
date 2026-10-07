# ADR-006 — Separate Capability Compilation, Qualification and Site Admission

Status: Accepted for v11.5.

## Decision

Artifact compilation produces a Declared candidate only. Evaluation/qualification,
release/package identity, profile conformance and site admission are separate
gates.

Qualification records carry evidence such as test suites, environment digest,
expected properties, known failures, evaluator identity and validity.

## Consequences

A generated skill/agent/API wrapper cannot self-promote to production use.
Admission can be suspended without rewriting qualification history.
