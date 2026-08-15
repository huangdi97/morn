// UI runtime smoke: loads the four Morn surfaces in headless Chromium,
// runs the BioLab E2E, and fails on any console/page error or uncaught exception.

import { chromium } from "playwright";

const BASE = process.env.UI_BASE ?? "http://127.0.0.1:5173";

const routes = ["/workbench", "/studio", "/console", "/hub"];
const L = (s) => s.toLowerCase();

const browser = await chromium.launch();
const page = await browser.newPage();
const errors = [];

page.on("console", (msg) => {
  if (msg.type() === "error") errors.push(`console.error: ${msg.text()}`);
});
page.on("pageerror", (err) => errors.push(`pageerror: ${err.message}`));

try {
  for (const route of routes) {
    errors.length = 0;
    await page.goto(`${BASE}${route}`, { waitUntil: "networkidle", timeout: 30000 });
    await page.waitForTimeout(500);
    const bodyText = await page.locator("body").innerText();
    if (!bodyText.includes("Morn")) errors.push(`route ${route}: missing Morn shell`);
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
    await page.getByRole("button", { name: /Run BioLab E2E/i }).click();
    await page.waitForTimeout(2500);
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
      { name: /Start Durable Run/i, check: (t) => t.includes("durable work runtime") && t.includes("running") },
      { name: /Run Replay \(drift\)/i, check: (t) => t.includes("replay reproduced") && t.includes("no") },
      { name: /Shadow Compare/i, check: (t) => t.includes("shadow readiness") },
      { name: /Evaluate \(approval missing\)/i, check: (t) => t.includes("evaluation decision") },
      { name: /Run Loop A/i, check: (t) => t.includes("loop a") && t.includes("approved=true") },
    ];
    for (const btn of buttons) {
      errors.length = 0;
      await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
      await page.getByRole("button", { name: btn.name }).click();
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
      if (!t.includes("solutionpackage manifest")) errors.push("studio manifest missing");
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
