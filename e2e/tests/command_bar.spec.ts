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

test.describe('CommandBar (Ctrl+G) GO navigation & ticker search', () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
  });

  test('Ctrl+G opens the bar, typing a ticker + DES routes to the Security Hub', async ({ page }) => {
    await page.goto('/portfolio');

    await page.keyboard.press('Control+g');
    const input = page.locator('.hf-command-bar__input');
    await expect(input).toBeFocused();
    await expect(page.locator('[role="combobox"]')).toHaveAttribute('aria-expanded', 'true');

    await input.fill('RELIANCE DES');
    await page.keyboard.press('Enter');
    await page.waitForURL(/.*\/equity\/security\/RELIANCE/);

    await expect(page.getByRole('tab', { name: 'Overview' })).toBeVisible();
  });

  test('typing a ticker shows the suggestions list before navigating', async ({ page }) => {
    await page.goto('/portfolio');

    await page.keyboard.press('Control+g');
    const input = page.locator('.hf-command-bar__input');
    await input.fill('RELIANCE');
    await expect(page.getByRole('listbox')).toBeVisible();

    await input.fill(' DES');
    await page.keyboard.press('Enter');
    await page.waitForURL(/.*\/equity\/security\/RELIANCE/);
  });

  test('an unsupported command shows feedback instead of navigating', async ({ page }) => {
    await page.goto('/portfolio');

    await page.keyboard.press('Control+g');
    const input = page.locator('.hf-command-bar__input');
    await input.fill('NOTAREALCOMMAND');
    await page.keyboard.press('Enter');

    // Remains on the portfolio page; the bar stays open with an error hint.
    await expect(page).toHaveURL('/portfolio');
    await expect(page.locator('[role="combobox"]')).toHaveAttribute('aria-expanded', 'true');
  });
});
