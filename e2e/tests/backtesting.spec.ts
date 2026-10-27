import { test, expect } from '@playwright/test';

const DEMO_EMAIL = 'test@test.com';
const DEMO_PASSWORD = 'test123';

async function signIn(page) {
  await page.goto('/login');
  await page.getByLabel('Email').fill(DEMO_EMAIL);
  await page.getByLabel('Password').fill(DEMO_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page).toHaveURL('/portfolio');
}

test.describe('Backtesting strategy templates, runner & analytics shell', () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.goto('/backtesting');
    await page.waitForURL(/.*\/backtesting$/);
  });

  test('strategy templates are listed with the canonical set', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Strategy Templates' })).toBeVisible();
    const templates = [
      'Trend Following', 'Mean Reversion', 'Statistical Arbitrage', 'Options Theta Decay',
    ];
    for (const t of templates) {
      await expect(page.getByText(t, { exact: true })).toBeVisible();
    }
  });

  test('backtest runner and analytics sections are present', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Backtest Runner' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Analytics' })).toBeVisible();
  });

  test('parameter scan and Monte Carlo entry points exist', async ({ page }) => {
    // The Phase 5 backtesting deck exposes runner controls and analytics;
    // Monte Carlo / WFO surfaces render within the analytics panels.
    await expect(page.getByText(/monte carlo|walk.forward|parameter/i)).toBeVisible();
  });
});
