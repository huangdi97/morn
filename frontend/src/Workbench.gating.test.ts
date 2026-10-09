import { describe, expect, it } from "vitest";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router-dom";
import type { V115ControlPlaneData } from "./api";
import {
  biolabEnabled,
  CanonicalWorkOverview,
  uiExtensionEnabledForProfiles,
  governedRealHarnessEnvironment,
  ValueAssessmentPanel,
  workEvidenceTrace,
} from "./pages/Workbench";

describe("domain-gated UI extension point", () => {
  it("zero-domain: no BioLab UI is rendered", () => {
    expect(biolabEnabled([])).toBe(false);
    expect(biolabEnabled(["generic"])).toBe(false);
  });

  it("enabled only when the backend advertises the biolab-reference pack", () => {
    expect(biolabEnabled(["biolab-reference"])).toBe(true);
  });
});

describe("declarative UI extension profile gate", () => {
  it("allows profile-neutral extensions but blocks mismatched profile declarations", () => {
    expect(uiExtensionEnabledForProfiles(null, [])).toBe(true);
    expect(
      uiExtensionEnabledForProfiles(
        "morn.factory.readonly@1.0.0",
        ["morn.factory.readonly@1.0.0"],
      ),
    ).toBe(true);
    expect(
      uiExtensionEnabledForProfiles(
        "morn.enterprise@1.0.0",
        ["morn.factory.readonly@1.0.0"],
      ),
    ).toBe(false);
    expect(uiExtensionEnabledForProfiles("morn.enterprise@1.0.0", [])).toBe(false);
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
    interop_bindings: [],
    external_task_observations: [],
    binding_migrations: [],
    durable_workflow_bindings: [],
    attempts: [],
    reconciliations: [],
    outcomes: [],
    acceptance_decisions: [],
    value_assessments: [],
    value_assessment_support: [],
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

  it("shows revoked RealSite support without rewriting a historical CustomerValidated assessment", () => {
    const state: V115ControlPlaneData = {
      ...base,
      outcomes: [
        {
          id: "outcome-1",
          work_package_id: "work:canonical-1",
          work_generation: 1,
          objective: "Customer delivery result",
          source_ref: "erp://delivery/42",
          evidence_refs: ["erp://delivery/42/receipt"],
        },
      ],
      acceptance_decisions: [
        {
          id: "decision-1",
          work_package_id: "work:canonical-1",
          work_generation: 1,
          disposition: "Accept",
          outcome_refs: ["outcome-1"],
          acting_role: "independent-reviewer",
          reason: "accepted",
        },
      ],
      value_assessments: [
        {
          id: "value-1",
          work_package_id: "work:canonical-1",
          work_generation: 1,
          outcome_ref: "outcome-1",
          acceptance_ref: "decision-1",
          evidence_class: "CustomerValidated",
          evidence_refs: ["customer://site-pilot/42"],
        },
      ],
      value_assessment_support: [
        {
          assessment_id: "value-1",
          subject: "value:work:canonical-1:g1:outcome-1",
          requires_real_site: true,
          currently_supported: false,
          current_real_site_state: "revoked",
          current_real_site_claim_id: "evidence-claim:revocation-1",
          current_real_site_evidence_refs: ["incident://revocation-1"],
        },
      ],
    };
    const html = render(state, null);
    expect(html).toContain("CustomerValidated");
    expect(html).toContain("Current RealSite support: revoked");
    expect(html).toContain("evidence-claim:revocation-1");
    expect(html).toContain("historical evidence, not a permanent entitlement");
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
        { id: "receipt-harness-1", execution_binding_ref: "binding-1", work_generation: 1, provider_ref: "deepseek-harness", outcome: "completed", session_id: "session-1", runtime_version: "1.2.3", runtime_digest: "sha256:live-one" },
        { id: "receipt-harness-old", execution_binding_ref: "binding-1", work_generation: 0, provider_ref: "deepseek-harness", outcome: "completed" },
        { id: "receipt-harness-other", execution_binding_ref: "binding-other", work_generation: 1, provider_ref: "pi", outcome: "completed" },
      ],
      interop_bindings: [
        { work_ref: "work:canonical-1", execution_binding_ref: "binding-1", capability_ref: "cap:mcp", endpoint: { protocol: "Mcp", endpoint_ref: "https://mcp.example.com" } },
        { work_ref: "work:foreign", execution_binding_ref: "binding-other", capability_ref: "cap:foreign", endpoint: { protocol: "A2a", endpoint_ref: "https://agent.example/a2a" } },
      ],
      external_task_observations: [
        { id: "task-observation-1", work_id: "work:canonical-1", work_generation: 1, execution_binding_ref: "binding-1", endpoint: { protocol: "Mcp", endpoint_ref: "https://mcp.example.com" }, snapshot: { mcp: { task_id: "task-1" } } },
        { id: "task-observation-unbound", work_id: "work:canonical-1", work_generation: 1, execution_binding_ref: "binding-other", endpoint: { protocol: "A2a", endpoint_ref: "https://agent.example/a2a" }, snapshot: { a2a: { task_id: "task-foreign" } } },
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
        { id: "outcome-1", work_package_id: "work:canonical-1", work_generation: 1, objective: "Validated review", source_ref: "cmms://plant-a", evidence_refs: ["receipt-1"] },
        { id: "outcome-old", work_package_id: "work:canonical-1", work_generation: 0, objective: "Old generation review", source_ref: "cmms://plant-a", evidence_refs: ["receipt-old"] },
        { id: "outcome-other", work_package_id: "work:foreign", work_generation: 1, objective: "Foreign review" },
      ],
      acceptance_decisions: [
        { id: "decision-1", work_package_id: "work:canonical-1", work_generation: 1, disposition: "Conditional", outcome_refs: ["outcome-1"], acting_role: "reviewer" },
        { id: "decision-old", work_package_id: "work:canonical-1", work_generation: 0, disposition: "Accept", outcome_refs: ["outcome-old"], acting_role: "reviewer" },
        { id: "decision-spoof", work_package_id: "work:canonical-1", work_generation: 1, disposition: "Accept", outcome_refs: ["outcome-other"] },
      ],
      value_assessments: [
        { id: "value-1", work_package_id: "work:canonical-1", work_generation: 1, outcome_ref: "outcome-1", acceptance_ref: "decision-1", evidence_class: "ObservedOperational", evidence_refs: ["metric://one"] },
        { id: "value-old", work_package_id: "work:canonical-1", work_generation: 0, outcome_ref: "outcome-old", acceptance_ref: "decision-old", evidence_class: "CustomerValidated", evidence_refs: ["customer://old"] },
        { id: "value-spoof", work_package_id: "work:canonical-1", work_generation: 1, outcome_ref: "outcome-other", acceptance_ref: "decision-spoof", evidence_class: "CustomerValidated", evidence_refs: ["customer://foreign"] },
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
    expect(trace.interopBindings).toHaveLength(1);
    expect(trace.externalTasks).toHaveLength(1);
    expect(trace.attempts).toHaveLength(1);
    expect(trace.reconciliations).toHaveLength(1);
    expect(trace.outcomes).toHaveLength(1);
    expect(trace.acceptances).toHaveLength(1);
    expect(trace.values).toHaveLength(1);
    expect(trace.evidence).toHaveLength(1);
    const html = render(state, null);
    expect(html).toContain("Validated review");
    expect(html).toContain("OutcomeUnknown");
    expect(html).toContain("Conditional");
    expect(html).toContain("ObservedOperational");
    expect(html).toContain("sha256:live-one");
    expect(html).toContain("https://mcp.example.com");
    expect(html).toContain("MCP task observation");
    expect(html).toContain("Value evidence refs: 1");
    expect(html).not.toContain("<b>CustomerValidated</b>");
    expect(html).not.toContain("Foreign review");
    expect(html).not.toContain("Old generation review");
    expect(html).not.toContain("cmms.foreign");
  });
});


describe("accepted outcome value UI", () => {
  it("requires an accepted Work, exact source-grounded outcome and Accept decision", () => {
    const control: V115ControlPlaneData = {
      condition_evidence: [],
      capability_resolutions: [],
      profile_conformance_attestations: [],
      work: [{
        id: "work:value",
        generation: 2,
        spec: { goal: "Measure delivery improvement", profile_ref: "morn.lite@1.0.0", required_conditions: [] },
        status: { phase: "Accepted", observed_generation: 2, conditions: [], active_binding: null },
      }],
      source_of_truth_bindings: [],
      execution_bindings: [],
      execution_manifests: [],
      execution_receipts: [],
    interop_bindings: [],
    external_task_observations: [],
      binding_migrations: [],
      durable_workflow_bindings: [],
      attempts: [],
      reconciliations: [],
      outcomes: [{
        id: "outcome:value",
        work_package_id: "work:value",
        work_generation: 2,
        objective: "Delivery improved",
        source_ref: "erp://delivery/42",
        evidence_refs: ["erp://delivery/42/receipt"],
      }],
      acceptance_decisions: [{
        id: "accept:value",
        work_package_id: "work:value",
        work_generation: 2,
        disposition: "Accept",
        outcome_refs: ["outcome:value"],
        acting_role: "customer-owner",
      }],
      value_assessments: [],
      note: "test",
    };
    const html = renderToStaticMarkup(
      createElement(MemoryRouter, null, createElement(ValueAssessmentPanel, {
        control,
        reload: () => undefined,
      })),
    );
    expect(html).toContain("Accepted outcome value");
    expect(html).toContain("Delivery improved");
    expect(html).toContain("Customer validated (requires RealSite)");
    expect(html).toContain("Persist value assessment");
  });
});

describe("real Harness environment gating", () => {
  const status = {
    harness_runtime: {
      dsh: {
        mode: "real",
        health: "initialized",
        configured_execution_environment_ref: "env://dsh/a",
      },
      pi: {
        mode: "real",
        health: "initialized",
        configured_execution_environment_ref: "env://pi/a",
      },
    },
  } as import("./api").V115Status;

  it("pins both real DSH and real Pi to their deployment-configured environment", () => {
    expect(governedRealHarnessEnvironment("deepseek-harness", status)).toBe("env://dsh/a");
    expect(governedRealHarnessEnvironment("pi", status)).toBe("env://pi/a");
    expect(governedRealHarnessEnvironment("morn-native", status)).toBeNull();
  });
});
