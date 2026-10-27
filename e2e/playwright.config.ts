import { defineConfig, devices } from '@playwright/test';

// Set E2E_BASE_URL to test an already-running server (no webServer is started).
const externalBaseURL = process.env.E2E_BASE_URL;

export default defineConfig({
  testDir: './tests',
  testMatch: /.*\.spec\.ts/,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'github' : 'list',

  use: {
    baseURL: externalBaseURL ?? 'http://127.0.0.1:8080',
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

  // Serve the release build via trunk (SPA fallback for deep links such as
  // /equity/ops). CI runs `trunk build --release` first, so this is a cache
  // hit; locally an already-running `trunk serve` is reused.
  webServer: externalBaseURL ? undefined : {
    command: 'trunk serve --release --no-autoreload',
    cwd: '..',
    url: 'http://127.0.0.1:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
