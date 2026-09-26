import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './src/test/setup.ts',
    // Playwright specs under e2e/ are not vitest suites; without this exclude
    // vitest collects them and fails on test.describe().
    exclude: ['**/node_modules/**', '**/dist/**', 'e2e/**'],
  },
});
