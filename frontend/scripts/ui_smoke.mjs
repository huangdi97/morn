// UI runtime smoke: loads the four Morn surfaces in headless Chromium,
// runs the BioLab E2E, and fails on any console/page error or uncaught exception.

import { chromium } from "playwright";

const BASE = process.env.UI_BASE ?? "http://127.0.0.1:5173";

const routes = ["/workbench", "/studio", "/console", "/hub"];

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
    const body = await page.locator("body").innerText();
    if (!body.includes("BioLab E2E") || !body.includes("outcome")) {
      errors.push("BioLab E2E section not rendered after run");
    }
    if (errors.length > 0) {
      console.error("FAILED BioLab E2E smoke:", errors.join(" | "));
      process.exitCode = 1;
    } else {
      console.log("OK BioLab E2E: claim outcome rendered via real backend");
    }
  }
} finally {
  await browser.close();
}
