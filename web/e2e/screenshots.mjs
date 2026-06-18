import { chromium } from '@playwright/test';

const BASE = 'http://localhost:5173';

async function main() {
  const browser = await chromium.launch({ headless: true });
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await ctx.newPage();
  const dir = 'e2e/screenshots/audit';

  // 进入 App
  await page.goto(BASE);
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForLoadState('networkidle');

  // 跳过欢迎页
  await page.locator('button', { hasText: '我有 API Key' }).click();
  await page.waitForTimeout(800);

  // 1. Workbench 空态
  await page.screenshot({ path: `${dir}/01-workbench-empty.png`, fullPage: true });
  console.log('01: Workbench 空态');

  // 2. 发几条消息后
  const ta = page.locator('textarea');
  const send = page.locator('footer.input-bar button');
  for (const msg of ['你好，Morn', '帮我写一个 Python 脚本', '今天天气怎么样', '什么是 Agent AI']){
    await ta.fill(msg); await send.click(); await page.waitForTimeout(200);
  }
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${dir}/02-workbench-chat.png`, fullPage: true });
  console.log('02: Workbench 聊天');

  // 3. Studio 各个 tab（evaluate 导航到 Studio）
  await page.evaluate(() => { const btns = document.querySelectorAll('.main-tabs button'); if (btns[1]) btns[1].click(); });
  await page.waitForTimeout(600);
  for (let i = 0; i < 8; i++) {
    const tabs = page.locator('.studio-tabs button');
    await tabs.nth(i).click();
    await page.waitForTimeout(400);
    await page.screenshot({ path: `${dir}/03-studio-tab-${i}.png`, fullPage: true });
    console.log(`03: Studio tab ${i}`);
  }

  // 4. Store
  await page.locator('.main-tabs button').nth(2).click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${dir}/04-store.png`, fullPage: true });
  console.log('04: Store');

  // 5. Console tabs（前几个代表性）
  await page.locator('.main-tabs button').nth(3).click();
  await page.waitForTimeout(500);
  for (const i of [0, 1, 2, 5, 8, 9, 10, 12, 15, 20]) {
    const tabs = page.locator('.console-tabs button');
    if (i < await tabs.count()) {
      await page.evaluate((idx) => {
        const tabs = document.querySelectorAll('.console-tabs button');
        if (tabs[idx]) tabs[idx].click();
      }, i);
      await page.waitForTimeout(300);
      await page.screenshot({ path: `${dir}/05-console-tab-${i}.png`, fullPage: true });
      console.log(`05: Console tab ${i}`);
    }
  }

  // 6. 设置弹窗
  await page.evaluate(() => {
    const btn = document.querySelector('header .settings-btn');
    if (btn) btn.click();
  });
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${dir}/06-settings.png`, fullPage: true });
  console.log('06: 设置');

  // 7. 小窗口
  await page.setViewportSize({ width: 480, height: 800 });
  await page.waitForTimeout(300);
  await page.evaluate(() => {
    const overlay = document.querySelector('.settings-overlay');
    if (overlay) overlay.style.display = 'none';
  });
  await page.waitForTimeout(200);
  await page.locator('.main-tabs button').first().click({ force: true });
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${dir}/07-mobile-view.png`, fullPage: true });
  console.log('07: 小窗口');

  await browser.close();
  console.log('\nDone!');
}

main().catch(console.error);
