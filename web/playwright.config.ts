import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 60000,
  expect: {
    timeout: 5000,
  },
  use: {
    baseURL: 'http://localhost:5173',
    headless: true,
    // PROVIDER: use the system Edge channel — the pinned chromium_headless_shell
    // build for @playwright/test 1.61 is not installed locally, and the
    // system Edge is a real browser available without a download.
    channel: 'msedge',
    screenshot: 'only-on-failure',
  },
  retries: 1,
  reporter: [
    ['list'],
    ['html', { output: 'e2e/report' }],
  ],
});
