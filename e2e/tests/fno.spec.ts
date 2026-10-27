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

test.describe('F&O option chain strike filtering & strategy payoff', () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
  });

  test('option chain default strike range is 15 and "All" widens the matrix', async ({ page }) => {
    await page.goto('/fno');
    await page.waitForURL(/.*\/fno$/);

    // Range toggle group: 10 / 15 / 20 / All; "15" is the default.
    await expect(page.getByRole('button', { name: '15' })).toHaveAttribute('aria-pressed', 'true');

    await page.getByRole('button', { name: 'All' }).click();
    await expect(page.getByRole('button', { name: 'All' })).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByRole('button', { name: '15' })).toHaveAttribute('aria-pressed', 'false');
  });

  test('strategy builder payoff curve and metrics render for a preset', async ({ page }) => {
    await page.goto('/fno/strategy');
    await page.waitForURL(/.*\/fno\/strategy$/);

    const presets = [
      'Bull Call Spread', 'Bear Put Spread', 'Long Straddle', 'Short Strangle',
      'Iron Condor', 'Iron Butterfly',
    ];
    for (const label of presets) {
      await expect(page.getByRole('button', { name: label })).toBeVisible();
    }

    await page.getByRole('button', { name: 'Iron Condor' }).click();
    await expect(page.getByRole('button', { name: 'Iron Condor' })).toHaveAttribute('aria-pressed', 'true');

    // The expiry payoff curve is rendered as an SVG; strategic metrics follow.
    await expect(page.getByRole('img', { name: 'Net payoff at expiry' })).toBeVisible();
    await expect(
      page.locator('text=/max profit|max loss|breakeven|net debit|net credit/i').first()
    ).toBeVisible();
  });
});
