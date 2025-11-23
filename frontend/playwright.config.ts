import { defineConfig, devices } from '@playwright/test';

/**
 * E2E test configuration for Poker Tracker frontend.
 * Tests run against a running instance of the full stack.
 *
 * Environment variables:
 * - BASE_URL: The URL of the frontend (default: http://localhost:8888)
 * - CI: Set in CI/Docker environments for stricter settings
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'list' : 'html',
  timeout: 30000,
  expect: {
    timeout: 10000,
  },
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:8888',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
