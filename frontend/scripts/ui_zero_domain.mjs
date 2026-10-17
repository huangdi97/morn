// Zero-domain runtime evidence: proves a server built without `domain-biolab`
// exposes no BioLab routes, advertises no domain packs, and never leaks BioLab
// UI into Workbench or Studio.
//
// This complements the in-process zero-domain E2E: it exercises the real HTTP
// router and the real browser surface.

import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.UI_BASE ?? "http://127.0.0.1:5173";
const API = process.env.MORN_API_BASE ?? "http://127.0.0.1:8090";
const screenshotsDir = process.env.UI_SCREENSHOT_DIR;
if (screenshotsDir) await mkdir(screenshotsDir, { recursive: true });

const failures = [];

async function api(path, init) {
  const response = await fetch(`${API}${path}`, init);
  let body = null;
  try {
    body = await response.json();
  } catch {
    body = null;
  }
  return { status: response.status, body };
}

const health = await api("/api/health");
if (health.status !== 200 || health.body?.status !== "ok") {
  failures.push(`zero-domain health failed: ${health.status} ${JSON.stringify(health.body)}`);
}

const workbench = await api("/api/workbench");
const domainPacks = workbench.body?.domain_packs;
if (!Array.isArray(domainPacks) || domainPacks.length !== 0) {
  failures.push(
    `zero-domain /api/workbench advertised domain packs: ${JSON.stringify(domainPacks)}`,
  );
}

for (const [method, path] of [
  ["POST", "/api/biolab/run"],
  ["GET", "/api/biolab/result"],
  ["GET", "/api/biolab/assets"],
  ["POST", "/api/biolab/loop-a"],
  ["POST", "/api/biolab/loop-c"],
]) {
  const result = await api(path, { method });
  if (result.status !== 404) {
    failures.push(`zero-domain ${method} ${path} returned ${result.status}, expected 404`);
  }
}

const browser = await chromium.launch();
const consoleErrors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(`console.error: ${message.text()}`);
  });
  page.on("pageerror", (error) => consoleErrors.push(`pageerror: ${error.message}`));

  await page.goto(`${BASE}/workbench`, { waitUntil: "networkidle", timeout: 30000 });
  await page.locator(".work-focus").waitFor({ state: "visible", timeout: 15000 });
  const workbenchMarkup = await page.content();
  if (/biolab/i.test(workbenchMarkup)) {
    failures.push("zero-domain Workbench DOM leaked BioLab UI");
  }
  const reference = page.locator('[data-testid="reference-tools"]');
  if ((await reference.count()) === 0) {
    failures.push("zero-domain Workbench must still expose collapsed engineering diagnostics");
  } else if (await reference.evaluate((node) => node.open)) {
    failures.push("zero-domain reference diagnostics should be collapsed by default");
  }
  if (screenshotsDir) {
    await page.screenshot({
      path: join(screenshotsDir, "workbench-zero-domain.png"),
      fullPage: true,
    });
  }

  await page.goto(`${BASE}/studio`, { waitUntil: "networkidle", timeout: 30000 });
  await page.getByRole("heading", { name: /Studio/i }).waitFor({ state: "visible" });
  const goalInput = page.locator(".builder-item", { hasText: "Goal:" }).locator("input");
  if ((await goalInput.count()) === 0) {
    failures.push("zero-domain Studio has no Goal input");
  } else if ((await goalInput.first().inputValue()) !== "Deliver a reviewed report") {
    failures.push(
      `zero-domain Studio default goal is not generic: ${await goalInput.first().inputValue()}`,
    );
  }
  const studioMarkup = await page.content();
  if (/biolab/i.test(studioMarkup)) {
    failures.push("zero-domain Studio DOM leaked BioLab UI");
  }
  if (screenshotsDir) {
    await page.screenshot({
      path: join(screenshotsDir, "studio-zero-domain.png"),
      fullPage: true,
    });
  }
} finally {
  await browser.close();
}

for (const error of consoleErrors) failures.push(`unexpected console error: ${error}`);

if (failures.length > 0) {
  console.error("Zero-domain runtime evidence failed:");
  for (const failure of failures) console.error(` - ${failure}`);
  process.exit(1);
}
console.log("Zero-domain runtime evidence OK: no routes, no packs, no BioLab UI");
