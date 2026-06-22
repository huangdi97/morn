import { chromium } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outputDir = path.resolve(__dirname, '../public/screenshots');

const BASE = 'http://localhost:5173';

const views = [
  { name: 'welcome-chat', url: BASE + '/', tab: 'workbench', selector: '.chat-area' },
  { name: 'studio-canvas', url: BASE + '/', tab: 'studio', selector: '.studio-view' },
  { name: 'store-browse', url: BASE + '/', tab: 'hub', selector: '.console-view' },
  { name: 'console-dashboard', url: BASE + '/', tab: 'console', selector: '.console-view', action: async (page) => {
    const tabs = page.locator('.console-tabs button');
    const count = await tabs.count();
    for (let i = 0; i < count; i++) {
      const text = await tabs.nth(i).textContent();
      if (text?.toLowerCase().includes('dashboard')) { await tabs.nth(i).click(); break; }
    }
  }},
  { name: 'console-topology', url: BASE + '/', tab: 'console', selector: '.console-view', action: async (page) => {
    const tabs = page.locator('.console-tabs button');
    const count = await tabs.count();
    for (let i = 0; i < count; i++) {
      const text = await tabs.nth(i).textContent();
      if (text?.toLowerCase().includes('topology')) { await tabs.nth(i).click(); break; }
    }
  }},
  { name: 'console-cost', url: BASE + '/', tab: 'console', selector: '.console-view', action: async (page) => {
    const tabs = page.locator('.console-tabs button');
    const count = await tabs.count();
    for (let i = 0; i < count; i++) {
      const text = await tabs.nth(i).textContent();
      if (text?.toLowerCase().includes('cost')) { await tabs.nth(i).click(); break; }
    }
  }},
];

async function capture() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

  for (const view of views) {
    await page.goto(view.url, { waitUntil: 'networkidle' });

    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForLoadState('networkidle');

    // navigate by clicking the appropriate main tab
    if (view.tab) {
      const mainTabs = page.locator('.main-tabs button');
      const count = await mainTabs.count();
      for (let i = 0; i < count; i++) {
        const text = await mainTabs.nth(i).textContent();
        if (text?.toLowerCase().includes(view.tab)) {
          await mainTabs.nth(i).click();
          break;
        }
      }
      await page.waitForTimeout(800);
    }

    if (view.action) {
      await view.action(page);
      await page.waitForTimeout(500);
    }

    try {
      await page.waitForSelector(view.selector, { timeout: 8000 });
    } catch {
      console.warn(`Selector "${view.selector}" not found for ${view.name}, taking full page screenshot`);
    }

    await page.screenshot({ path: path.join(outputDir, view.name + '.png'), fullPage: false });
    console.log(`Captured: ${view.name}`);
  }

  await browser.close();
  console.log('\nDone! Screenshots saved to', outputDir);
}

capture().catch(console.error);