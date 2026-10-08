import { describe, expect, it } from "vitest";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router-dom";
import type { V115ControlPlaneData } from "./api";
import { biolabEnabled, CanonicalWorkOverview } from "./pages/Workbench";

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
});
