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

test.describe('Operations Console emergency kill switches', () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.goto('/equity/ops');
    await page.waitForURL(/.*\/equity\/ops$/);
  });

  test('the ops dashboard shows all five desks', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Operations Console' })).toBeVisible();
    const desks = ['Trader Desk', 'Quant Desk', 'PM Desk', 'Risk Desk', 'System Desk'];
    for (const d of desks) {
      await expect(page.getByRole('heading', { name: d })).toBeVisible();
    }
  });

  test('emergency kill switches are present', async ({ page }) => {
    const switches = [
      'Panic Sell All', 'Cancel All Pending', 'Halt Trading Engine', 'Safe Mode Toggle',
    ];
    for (const s of switches) {
      await expect(page.getByRole('button', { name: s })).toBeVisible();
    }
  });

  test('activating a kill switch does not crash the shell', async ({ page }) => {
    await page.getByRole('button', { name: 'Panic Sell All' }).click();
    await page.getByRole('button', { name: 'Cancel All Pending' }).click();
    await page.getByRole('button', { name: 'Halt Trading Engine' }).click();
    await page.getByRole('button', { name: 'Safe Mode Toggle' }).click();

    // The chrome and the ops page remain mounted after the interactions.
    await expect(page.getByRole('heading', { name: 'Operations Console' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Panic Sell All' })).toBeVisible();
  });
});
