// UI runtime smoke: loads the four Morn surfaces in headless Chromium,
// runs the BioLab E2E, and fails on any console/page error or uncaught exception.

import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.UI_BASE ?? "http://127.0.0.1:5173";

const routes = ["/workbench", "/studio", "/console", "/hub"];
const L = (s) => s.toLowerCase();

const screenshotsDir = process.env.UI_SCREENSHOT_DIR;
if (screenshotsDir) await mkdir(screenshotsDir, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];

async function expandReferenceDiagnostics() {
  const details = page.locator("details.workbench-reference");
  if (await details.count()) {
    await details.locator("summary").click();
  }
}

page.on("console", (msg) => {
  if (msg.type() === "error") errors.push(`console.error: ${msg.text()}`);
});
page.on("pageerror", (err) => errors.push(`pageerror: ${err.message}`));

try {
  for (const route of routes) {
    errors.length = 0;
    await page.goto(`${BASE}${route}`, { waitUntil: "networkidle", timeout: 30000 });
    await page.waitForTimeout(500);
    if (screenshotsDir) {
      const surface = route.slice(1);
      await page.screenshot({ path: join(screenshotsDir, `${surface}-desktop.png`), fullPage: true });
      await page.setViewportSize({ width: 390, height: 844 });
      await page.screenshot({ path: join(screenshotsDir, `${surface}-mobile.png`), fullPage: true });
      const documentWidth = await page.evaluate(() => document.documentElement.scrollWidth);
      if (documentWidth > 390) errors.push(`route ${route}: mobile horizontal overflow ${documentWidth}px > 390px`);
      await page.setViewportSize({ width: 1280, height: 800 });
    }
    const bodyText = await page.locator("body").innerText();
    if (!bodyText.includes("Morn")) errors.push(`route ${route}: missing Morn shell`);
    if (route === "/hub" && !(await page.getByRole("heading", { name: "v11.5 Capability Supply Chain" }).isVisible())) {
      errors.push("hub: governed capability supply-chain section is not visible");
    }
    if (route === "/hub" && !(await page.getByRole("heading", { name: "Provider Fabric" }).isVisible())) {
      errors.push("hub: canonical Provider Fabric registry is not visible");
    }
    if (route === "/hub" && !(await page.getByRole("heading", { name: "Registry projections" }).isVisible())) {
      errors.push("hub: declared-only registry projections are not visible");
    }
    if (route === "/hub") {
      const hubText = await page.locator(".hub-provider-fabric").innerText();
      if (!hubText.includes("deepseek-harness") || !hubText.includes("pi")) {
        errors.push("hub: DSH/Pi provider registry entries are missing");
      }
      const discoveryText = await page.locator(".hub-discovery").innerText();
      const discoveryTextNormalized = discoveryText.toLowerCase();
      if (!discoveryTextNormalized.includes("declared metadata") || !discoveryTextNormalized.includes("business truth")) {
        errors.push("hub: discovery projection boundary is not explicit");
      }
      const legacy = page.locator("details.hub-reference-details");
      if (!(await legacy.count())) {
        errors.push("hub: reference catalog disclosure is missing");
      } else if (await legacy.evaluate((node) => node.open)) {
        errors.push("hub: legacy/reference catalogs should be collapsed by default");
      }
    }
    if (route === "/console") {
      const legacy = page.locator("details.console-reference-details");
      if (!(await legacy.count())) {
        errors.push("console: legacy diagnostics disclosure is missing");
      } else if (await legacy.evaluate((node) => node.open)) {
        errors.push("console: legacy/reference diagnostics should be collapsed by default");
      }
      try {
        await page.getByText("Harness / Runtime Health", { exact: true }).waitFor({
          state: "visible",
          timeout: 5000,
        });
        await page.getByText("v11.5 Persisted Control Records", { exact: true }).waitFor({
          state: "visible",
          timeout: 5000,
        });
      } catch {
        errors.push("console: canonical v11.5 provider/control health must remain visible");
      }
    }
    if (route === "/studio") {
      const compilerGoal = page.locator(".studio-compiler-goal");
      if (!(await compilerGoal.count()) || !(await compilerGoal.innerText()).includes("Deliver a reviewed report")) {
        errors.push("studio: compiler must reuse the goal defined in the primary creator step");
      }
      const advanced = page.locator("details.studio-capability-details");
      if (!(await advanced.count())) {
        errors.push("studio: governed advanced capability importer is missing");
      } else {
        if (await advanced.evaluate((node) => node.open)) {
          errors.push("studio: advanced raw asset imports must be collapsed by default");
        }
        await advanced.locator("summary").click();
        if (!(await page.getByRole("button", { name: "Compile OpenAPI" }).isVisible())) {
          errors.push("studio: expanded artifact candidate compiler is unavailable");
        }
        if (!(await page.getByRole("button", { name: "Compile Model Manifest" }).isVisible())) {
          errors.push("studio: model artifact compiler is unavailable");
        }
        if (!(await page.getByRole("button", { name: "Compile Workflow Manifest" }).isVisible())) {
          errors.push("studio: workflow artifact compiler is unavailable");
        }
        await page.getByRole("button", { name: "Compile Model Manifest" }).click();
        await page.getByText("Model stage").waitFor({ state: "visible", timeout: 10000 });
        if (!(await page.locator("body").innerText()).includes("Declared")) {
          errors.push("studio: model compiler did not preserve Declared candidate state");
        }
        await page.getByRole("button", { name: "Compile Workflow Manifest" }).click();
        await page.getByText("Workflow stage").waitFor({ state: "visible", timeout: 10000 });
        if (!(await page.locator("body").innerText()).includes("Workflow admission")) {
          errors.push("studio: workflow compiler did not return governed admission state");
        }
        await advanced.locator("summary").click();
      }
    }
    if (route === "/workbench" && !(await page.getByRole("heading", { name: "Work is the unit of coordination" }).isVisible())) {
      errors.push("workbench: canonical Work-first overview is not visible");
    }
    if (
      route === "/workbench" &&
      !(await page.getByRole("heading", { name: "Independent outcome review" }).isVisible())
    ) {
      errors.push("workbench: independent source-grounded outcome review is not visible");
    }
    if (route === "/workbench" && await page.locator("details.workbench-reference").evaluate((node) => node.open)) {
      errors.push("workbench: legacy diagnostics should be collapsed by default");
    }
    if (errors.length > 0) {
      console.error(`FAILED ${route}:`, errors.join(" | "));
      process.exitCode = 1;
      break;
    }
    console.log(`OK ${route}: ${(await page.title()) || "Morn"} loaded, no console errors`);
  }

  if (process.exitCode !== 1) {
    // BioLab E2E via the real backend through the UI button.
    errors.length = 0;
    await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
    await expandReferenceDiagnostics();
    await page.getByRole("button", { name: /Run BioLab E2E/i }).click();
    await page.waitForTimeout(2500);
    if (screenshotsDir) {
      await page.screenshot({ path: join(screenshotsDir, "workbench-biolab-after.png"), fullPage: true });
    }
    const body = (await page.locator("body").innerText()).toLowerCase();
    if (!body.includes("biolab e2e") || !body.includes("outcome")) {
      errors.push("BioLab E2E section not rendered after run");
    }
    if (errors.length > 0) {
      console.error("FAILED BioLab E2E smoke:", errors.join(" | "));
      process.exitCode = 1;
    } else {
      console.log("OK BioLab E2E: claim outcome rendered via real backend");
    }

    // Goal 2 v0.2 interactions: durable run, replay/shadow/eval, loops.
    const buttons = [
      { name: /Start Durable Run/i, check: (t) => t.includes("durable workflow provider") && t.includes("running") },
      { name: /Run Replay \(drift\)/i, check: (t) => t.includes("replay reproduced") && t.includes("no") },
      { name: "Shadow Compare", exact: true, check: (t) => t.includes("shadow readiness") },
      { name: /Evaluate \(approval missing\)/i, check: (t) => t.includes("evaluation decision") },
      { name: /Run Loop A/i, check: (t) => t.includes("loop a") && t.includes("approved=true") },
    ];
    for (const btn of buttons) {
      errors.length = 0;
      await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
      await expandReferenceDiagnostics();
      const opts = btn.exact ? { name: btn.name, exact: true } : { name: btn.name };
      await page.getByRole("button", opts).click();
      await page.waitForTimeout(3000);
      const t = (await page.locator("body").innerText()).toLowerCase();
      if (!btn.check(t)) errors.push(`button ${btn.name} result not rendered; body sample: ${t.slice(0, 300)}`);
      if (errors.length > 0) {
        console.error(`FAILED ${btn.name}:`, errors.join(" | "));
        process.exitCode = 1;
        break;
      }
      console.log(`OK ${btn.name}`);
    }

    // Goal 3 v0.3 interactions: evolution, distillation, managed work, replacement.
    const g3buttons = [
      { name: /Detect Patterns & Candidates/i, check: (t) => t.includes("patterns") && t.includes("distillation") },
      { name: /Distill QC Step/i, check: (t) => t.includes("distillation regression") && t.includes("passed=true") },
      { name: /Certify & Start Managed Work/i, check: (t) => t.includes("managed work") && t.includes("running") },
      { name: /Shadow Compare Baseline vs Candidate/i, check: (t) => t.includes("meets critical") && t.includes("yes") },
    ];
    for (const btn of g3buttons) {
      errors.length = 0;
      await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
      await expandReferenceDiagnostics();
      await page.getByRole("button", { name: btn.name }).click();
      await page.waitForTimeout(2500);
      const t = (await page.locator("body").innerText()).toLowerCase();
      if (!btn.check(t)) errors.push(`button ${btn.name} result not rendered; sample: ${t.slice(0, 200)}`);
      if (errors.length > 0) {
        console.error(`FAILED ${btn.name}:`, errors.join(" | "));
        process.exitCode = 1;
        break;
      }
      console.log(`OK ${btn.name}`);
    }

    // Studio compiler flow: run -> approve -> manifest.
    if (process.exitCode !== 1) {
      errors.length = 0;
      await page.goto(`${BASE}/studio`, { waitUntil: "networkidle", timeout: 30000 });
      await page.getByRole("button", { name: /Run Compiler/i }).click();
      await page.waitForTimeout(3000);
      let t = (await page.locator("body").innerText()).toLowerCase();
      if (!t.includes("workgraph") || !t.includes("validation")) errors.push("studio compiler output missing");
      await page.getByRole("button", { name: /Approve & Compile/i }).click();
      await page.waitForTimeout(1500);
      t = (await page.locator("body").innerText()).toLowerCase();
      if (screenshotsDir) {
        await page.screenshot({ path: join(screenshotsDir, "studio-compiled-after.png"), fullPage: true });
      }
      if (!t.includes("solutionpackage manifest")) errors.push("studio manifest missing");
      await page.getByRole("button", { name: "Instantiate Work", exact: true }).click();
      await page.locator('[role="status"]').filter({ hasText: "Readiness gates" }).waitFor();
      t = (await page.locator("body").innerText()).toLowerCase();
      if (!t.includes("no — explicit gates remain")) {
        errors.push("Studio: Work instantiation skipped execution readiness gates");
      }
      await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
      await page.waitForTimeout(500);
      const focusText = await page.locator(".work-focus").innerText();
      if (!focusText.includes("Deliver a reviewed report") || !focusText.includes("Proposed")) {
        errors.push("Workbench: Studio-instantiated Work is not shown as canonical state");
      }
      if (screenshotsDir) {
        await page.screenshot({ path: join(screenshotsDir, "workbench-after-studio-instantiation.png"), fullPage: true });
      }
      if (errors.length > 0) {
        console.error("FAILED studio compiler:", errors.join(" | "));
        process.exitCode = 1;
      } else {
        console.log("OK studio compiler flow (run -> approve -> manifest)");
      }
    }
  }
} finally {
  await browser.close();
}
