import { describe, expect, it } from "vitest";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router-dom";
import type { V115ControlPlaneData } from "./api";
import { biolabEnabled, CanonicalWorkOverview, workEvidenceTrace } from "./pages/Workbench";

describe("domain-gated UI extension point", () => {
  it("zero-domain: no BioLab UI is rendered", () => {
    expect(biolabEnabled([])).toBe(false);
    expect(biolabEnabled(["generic"])).toBe(false);
  });

  it("enabled only when the backend advertises the biolab-reference pack", () => {
    expect(biolabEnabled(["biolab-reference"])).toBe(true);
  });
});

describe("canonical Work remains a separate product surface", () => {
  const base: V115ControlPlaneData = {
    condition_evidence: [],
    capability_resolutions: [],
    profile_conformance_attestations: [],
    work: [{
      id: "work:canonical-1",
      generation: 1,
      spec: {
        goal: "Review CNC-17 delivery impact",
        profile_ref: "morn.factory.readonly@1.0.0",
        required_conditions: ["SourceOfTruthBound"],
      },
      status: {
        phase: "Blocked",
        observed_generation: 1,
        conditions: [{
          condition_type: "SourceOfTruthBound",
          status: "False",
          reason: "Not yet witnessed",
        }],
        active_binding: null,
      },
    }],
    source_of_truth_bindings: [],
    execution_bindings: [],
    execution_manifests: [],
    execution_receipts: [],
    binding_migrations: [],
    durable_workflow_bindings: [],
    attempts: [],
    reconciliations: [],
    outcomes: [],
    acceptance_decisions: [],
    value_assessments: [],
    note: "reference implementation",
  };

  const render = (control: V115ControlPlaneData | null, error: string | null) =>
    renderToStaticMarkup(
      createElement(
        MemoryRouter,
        null,
        createElement(CanonicalWorkOverview, { control, error }),
      ),
    );

  it("shows real persisted Work and unresolved evidence without inventing acceptance", () => {
    const html = render(base, null);
    expect(html).toContain("Review CNC-17 delivery impact");
    expect(html).toContain("SourceOfTruthBound");
    expect(html).toContain("Not bound");
    expect(html).toContain("Blocked");
    expect(html).not.toContain("Accepted");
  });

  it("shows an explicit canonical failure rather than legacy health as truth", () => {
    expect(render(null, "backend is unavailable")).toContain(
      "Canonical Work data is unavailable: backend is unavailable",
    );
    expect(render({ ...base, work: [] }, null)).toContain(
      "No canonical Work has been persisted yet.",
    );
  });

  it("correlates Work receipts and acceptance by binding, outcome and generation", () => {
    const state: V115ControlPlaneData = {
      ...base,
      capability_resolutions: [
        { id: "resolution-1", work_ref: "work:canonical-1", work_generation: 1, source_solution_ref: "solution://one@1.0.0" },
        { id: "resolution-old", work_ref: "work:canonical-1", work_generation: 0 },
        { id: "resolution-other", work_ref: "work:foreign", work_generation: 1 },
      ],
      execution_bindings: [
        { id: "binding-1", work_id: "work:canonical-1" },
        { id: "binding-other", work_id: "work:foreign" },
      ],
      execution_receipts: [
        { id: "receipt-harness-1", execution_binding_ref: "binding-1", work_generation: 1, provider_ref: "deepseek-harness", outcome: "completed", session_id: "session-1" },
        { id: "receipt-harness-old", execution_binding_ref: "binding-1", work_generation: 0, provider_ref: "deepseek-harness", outcome: "completed" },
        { id: "receipt-harness-other", execution_binding_ref: "binding-other", work_generation: 1, provider_ref: "pi", outcome: "completed" },
      ],
      attempts: [
        { id: "attempt-1", binding_id: "binding-1", action: "cmms.create", state: "OutcomeUnknown", business_key: "work:canonical-1:create" },
        { id: "attempt-other", binding_id: "binding-other", action: "cmms.foreign", state: "Verified" },
      ],
      reconciliations: [
        { id: "reconcile-1", attempt_id: "attempt-1" },
        { id: "reconcile-other", attempt_id: "attempt-other" },
      ],
      outcomes: [
        { id: "outcome-1", work_package_id: "work:canonical-1", objective: "Validated review", source_ref: "cmms://plant-a", evidence_refs: ["receipt-1"] },
        { id: "outcome-other", work_package_id: "work:foreign", objective: "Foreign review" },
      ],
      acceptance_decisions: [
        { id: "decision-1", work_package_id: "work:canonical-1", disposition: "Conditional", outcome_refs: ["outcome-1"], acting_role: "reviewer" },
        { id: "decision-spoof", work_package_id: "work:canonical-1", disposition: "Accept", outcome_refs: ["outcome-other"] },
      ],
      condition_evidence: [
        { id: "witness-1", work_ref: "work:canonical-1", work_generation: 1, condition_type: "SourceOfTruthBound", satisfied: true, producer_ref: "cmms://plant-a" },
        { id: "witness-old", work_ref: "work:canonical-1", work_generation: 0, condition_type: "SourceOfTruthBound", satisfied: true },
      ],
    };
    const trace = workEvidenceTrace(state, "work:canonical-1", 1);
    expect(trace.resolutions).toHaveLength(1);
    expect(trace.bindings).toHaveLength(1);
    expect(trace.receipts).toHaveLength(1);
    expect(trace.attempts).toHaveLength(1);
    expect(trace.reconciliations).toHaveLength(1);
    expect(trace.outcomes).toHaveLength(1);
    expect(trace.acceptances).toHaveLength(1);
    expect(trace.evidence).toHaveLength(1);
    const html = render(state, null);
    expect(html).toContain("Validated review");
    expect(html).toContain("OutcomeUnknown");
    expect(html).toContain("Conditional");
    expect(html).not.toContain("Foreign review");
    expect(html).not.toContain("cmms.foreign");
  });
});
