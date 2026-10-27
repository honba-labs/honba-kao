import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: /.*\.spec\.ts/,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'github' : 'list',

  use: {
    baseURL: 'http://127.0.0.1:8080',
    timezoneId: 'Asia/Kolkata',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: devices['Desktop Chrome'],
    },
  ],

  // The honba-kao app is served separately (see e2e/package.json notes):
  //   trunk build && python3 -m http.server 8080 --directory dist
  // webServer is intentionally omitted because honba-kao-app has pre-existing
  // compilation errors that prevent `trunk build` today (see plan.md Phase 8.2 note).
});
