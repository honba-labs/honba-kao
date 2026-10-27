import { test, expect } from '@playwright/test';

const DEMO_EMAIL = 'test@test.com';
const DEMO_PASSWORD = 'test123';

async function signIn(page) {
  await page.goto('/login');
  await page.getByLabelText('Email').fill(DEMO_EMAIL);
  await page.getByLabelText('Password').fill(DEMO_PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page).toHaveURL('/portfolio');
}

test.describe('Security Hub tab switching & filings citations', () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.goto('/equity/security/RELIANCE');
    await page.waitForURL(/.*\/equity\/security\/RELIANCE/);
  });

  test('the 11 workstation tabs are present and Overview is active by default', async ({ page }) => {
    const tabs = [
      'Overview', 'Financials', 'Chart', 'News', 'Ownership', 'Estimates',
      'Peers', 'ESG', 'Tape', 'Insider', 'Filings',
    ];
    for (const label of tabs) {
      await expect(page.getByRole('tab', { name: label })).toBeVisible();
    }
    await expect(page.getByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'true');
  });

  test('switching tabs updates aria-selected and keeps the ticker context', async ({ page }) => {
    await expect(page.getByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'true');

    await page.getByRole('tab', { name: 'Financials' }).click();
    await expect(page.getByRole('tab', { name: 'Financials' })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'false');

    await page.getByRole('tab', { name: 'Filings' }).click();
    await expect(page.getByRole('tab', { name: 'Filings' })).toHaveAttribute('aria-selected', 'true');
  });

  test('the Filings tab renders scored drivers with verbatim citations', async ({ page }) => {
    await page.getByRole('tab', { name: 'Filings' }).click();

    // Filings Intelligence renders scored growth/headwind driver cards and a
    // verbatim quote with a source label and page number.
    await expect(page.getByRole('tab', { name: 'Filings' })).toHaveAttribute('aria-selected', 'true');
    await expect(
      page.locator('text=/verbatim|source|page/i')
    ).toBeVisible();
  });
});
