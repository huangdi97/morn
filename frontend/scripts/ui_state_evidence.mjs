// UI state evidence: deterministic browser renders of canonical Work states
// that the happy-path four-surface smoke does not produce.
//
// Only /api/v115/control-plane is mocked; every other endpoint still comes from
// the running Morn server. All fixture material below is explicitly test data
// and must never be read as Outcome, Acceptance, RealSite or CustomerValidated
// evidence.

import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.UI_BASE ?? "http://127.0.0.1:5173";
const screenshotsDir = process.env.UI_SCREENSHOT_DIR;
if (screenshotsDir) await mkdir(screenshotsDir, { recursive: true });

const WORK_ID = "work:ui-state-fixture";
const OUTCOME_ID = "outcome:ui-state-fixture";

const work = (phase, conditions = []) => ({
  id: WORK_ID,
  generation: 1,
  spec: {
    goal: `Reference ${phase.toLowerCase()} Work`,
    profile_ref: "morn.lite@1.0.0",
    site_ref: null,
    required_conditions: [],
  },
  status: {
    phase,
    observed_generation: 1,
    conditions,
    active_binding: null,
    active_bindings: [],
  },
});

const outcome = {
  id: OUTCOME_ID,
  work_package_id: WORK_ID,
  work_generation: 1,
  objective: "Reference outcome observed by a deployment-attested source",
  source_kind: "ExternalSystem",
  source_ref: "fixture-authority://orders/42",
  source_binding_ref: "source-binding:ui-state-fixture",
  source_attestation_ref: "observation-attestation:ui-state-fixture",
  fact_type: "delivery.status",
  observed_facts: { status: "rejected-by-review" },
  evidence_refs: ["fixture-authority://orders/42/receipt"],
};

const rejection = {
  id: "acceptance:ui-state-fixture",
  work_package_id: WORK_ID,
  work_generation: 1,
  acceptance_spec_id: "acceptance-spec:ui-state-fixture",
  disposition: "Reject",
  outcome_refs: [OUTCOME_ID],
  evidence_refs: ["fixture-review://ticket-7"],
  decided_by: "principal:ui-independent-reviewer",
  acting_role: "independent-reviewer",
  reason: "reference UI state fixture: independent review rejected the observed outcome",
};

const control = (phase, extra = {}, conditions = []) => ({
  work: [work(phase, conditions)],
  condition_evidence: [],
  capability_resolutions: [],
  profile_conformance_attestations: [],
  source_of_truth_bindings: [],
  execution_bindings: [],
  execution_manifests: [],
  execution_receipts: [],
  execution_events: [],
  executor_outcome_reconciliations: [],
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
  note: "Playwright UI state fixture only; never business evidence",
  ...extra,
});

const failures = [];
const consoleErrors = [];

async function newPage() {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const pageErrors = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") pageErrors.push(`console.error: ${msg.text()}`);
  });
  page.on("pageerror", (error) => pageErrors.push(`pageerror: ${error.message}`));
  return { page, pageErrors };
}

async function screenshot(page, name) {
  if (screenshotsDir) {
    await page.screenshot({ path: join(screenshotsDir, name), fullPage: true });
  }
}

async function routeControlPlane(page, body, status = 200) {
  await page.route("**/api/v115/control-plane", (route) =>
    route.fulfill({
      status,
      contentType: "application/json",
      body: JSON.stringify(body),
    }),
  );
}

async function expectVisible(page, text) {
  await page.getByText(text, { exact: false }).first().waitFor({ state: "visible", timeout: 15000 });
}

const browser = await chromium.launch();

try {
  // 1. Loading state: hold the canonical response open until the loading copy
  //    has been rendered and captured.
  {
    const { page, pageErrors } = await newPage();
    let release;
    const gate = new Promise((resolve) => (release = resolve));
    await page.route("**/api/v115/control-plane", async (route) => {
      await gate;
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(control("Proposed")),
      });
    });
    await page.goto(`${BASE}/workbench`, { waitUntil: "domcontentloaded", timeout: 30000 });
    await expectVisible(page, "Loading persisted Work state");
    await screenshot(page, "workbench-loading-state.png");
    release();
    await page.getByTestId("work-lifecycle-guide").waitFor({ state: "visible", timeout: 15000 });
    consoleErrors.push(...pageErrors);
    await page.close();
    console.log("OK workbench loading state");
  }

  // 2. Reconciling state: the canonical Work is blocked on an unknown executor
  //    outcome; no execution may be offered as if retry were safe.
  {
    const { page, pageErrors } = await newPage();
    await routeControlPlane(
      page,
      control("Reconciling", {}, [
        {
          condition_type: "ExecutorOutcomeKnown",
          status: "False",
          reason: "executor settlement unknown; reconcile before any retry",
        },
      ]),
    );
    await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
    await page.getByTestId("work-lifecycle-guide").waitFor({ state: "visible", timeout: 15000 });
    const body = await page.locator("body").innerText();
    if (!body.includes("Reconciling")) {
      failures.push("reconciling fixture: Work phase is not rendered");
    }
    if (!/reconcil/i.test(body)) {
      failures.push("reconciling fixture: no reconciliation guidance is rendered");
    }
    if (!body.includes("Reconcile the unknown executor outcome")) {
      failures.push("reconciling fixture: Next instruction must reconcile before retry");
    }
    const execute = page.getByRole("button", { name: /Execute bound E0 capability/i });
    if ((await execute.count()) > 0 && (await execute.first().isEnabled())) {
      failures.push("reconciling fixture: blind E0 execution must not be enabled");
    }
    if (body.includes("Accepted outcome value")) {
      failures.push("reconciling fixture: value evidence must not appear before accepted outcome");
    }
    await screenshot(page, "workbench-reconciling-fixture.png");
    consoleErrors.push(...pageErrors);
    await page.close();
    console.log("OK workbench reconciling state");
  }

  // 3. Rejected state: a terminal independent rejection must never be rendered
  //    as accepted outcome value or customer value.
  {
    const { page, pageErrors } = await newPage();
    await routeControlPlane(
      page,
      control(
        "Rejected",
        { outcomes: [outcome], acceptance_decisions: [rejection] },
        [
          {
            condition_type: "OutcomeObservation",
            status: "True",
            reason: "fixture deployment-attested source outcome",
          },
          {
            condition_type: "IndependentAcceptance",
            status: "True",
            reason: "fixture independent reviewer rejected the outcome",
          },
          {
            condition_type: "AcceptedOutcomeSemantics",
            status: "False",
            reason: "independent review rejected the observed outcome",
          },
        ],
      ),
    );
    await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
    await page.getByTestId("work-lifecycle-guide").waitFor({ state: "visible", timeout: 15000 });
    const body = await page.locator("body").innerText();
    if (!body.includes("Rejected")) {
      failures.push("rejected fixture: Work phase is not rendered");
    }
    if (body.includes("Accepted outcome value")) {
      failures.push("rejected fixture: rejected Work must not show accepted outcome value");
    }
    if (body.includes("CustomerValidated")) {
      failures.push("rejected fixture: rejected Work must not show customer-validated value");
    }
    if ((await page.locator(".work-lifecycle-current").count()) > 0) {
      failures.push("rejected fixture: terminal Work must not point at an actionable current stage");
    }
    if (!body.includes("independent review rejected this outcome")) {
      failures.push("rejected fixture: Next instruction must explain the terminal rejection");
    }
    const acceptanceStage = page.locator(".work-lifecycle-stage").nth(4);
    const acceptanceClass = (await acceptanceStage.getAttribute("class")) ?? "";
    if (!acceptanceClass.includes("work-lifecycle-complete")) {
      failures.push("rejected fixture: an independent Reject decision must count as completed review");
    }
    await screenshot(page, "workbench-rejected-fixture.png");
    consoleErrors.push(...pageErrors);
    await page.close();
    console.log("OK workbench rejected state");
  }

  // 4. Error state: canonical control-plane failure must stay visible and must
  //    never be papered over by legacy diagnostics.
  {
    const { page, pageErrors } = await newPage();
    await routeControlPlane(page, { error: "fixture control-plane unavailable" }, 500);
    await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
    const alert = page.getByRole("alert");
    await alert.first().waitFor({ state: "visible", timeout: 15000 });
    const alertText = await alert.first().innerText();
    if (!alertText.includes("Canonical Work data is unavailable")) {
      failures.push("error fixture: canonical failure is not surfaced as an alert");
    }
    const body = await page.locator("body").innerText();
    if (!body.includes("Legacy diagnostics cannot establish canonical Work truth")) {
      failures.push("error fixture: legacy diagnostics may not substitute for canonical truth");
    }
    await screenshot(page, "workbench-canonical-error.png");
    // A 500 resource is the expected failure under test; other console errors
    // would still indicate a UI defect.
    consoleErrors.push(
      ...pageErrors.filter(
        (entry) => !entry.includes("500") && !entry.includes("Failed to load resource"),
      ),
    );
    await page.close();
    console.log("OK workbench canonical error state");
  }
} finally {
  await browser.close();
}

for (const entry of consoleErrors) {
  failures.push(`unexpected console error: ${entry}`);
}
if (failures.length > 0) {
  console.error("UI state evidence failed:");
  for (const failure of failures) console.error(` - ${failure}`);
  process.exit(1);
}
console.log("UI state evidence OK: loading / reconciling / rejected / canonical error");
