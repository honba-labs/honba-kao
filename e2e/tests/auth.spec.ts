import { test, expect } from '@playwright/test';

const DEMO_EMAIL = 'test@test.com';
const DEMO_PASSWORD = 'test123';

test.describe('auth flow & protected route redirects', () => {
  test('unauthenticated guest at / sees the marketing landing (not the login shell)', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL('/');
    await expect(page).toHaveTitle(/honba/);
    // The login form must not be present on the marketing landing.
    await expect(page.getByLabel('Email')).not.toBeVisible();
  });

  test('guest accessing a gated workspace route is redirected to /login', async ({ page }) => {
    await page.goto('/equity/ops');
    await expect(page).toHaveURL(/.*\/login/);
    await expect(page.getByLabel('Email')).toBeVisible();
    await expect(page.getByLabel('Password')).toBeVisible();
  });

  test('demo sign-in redirects to the portfolio landing', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel('Email').fill(DEMO_EMAIL);
    await page.getByLabel('Password').fill(DEMO_PASSWORD);
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL('/portfolio');
  });

  test('signed-in user can reach gated workspace routes', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel('Email').fill(DEMO_EMAIL);
    await page.getByLabel('Password').fill(DEMO_PASSWORD);
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL('/portfolio');

    await page.goto('/equity/ops');
    await expect(page.getByRole('heading', { name: 'Operations Console' })).toBeVisible();
  });

  test('signing out returns the guest to the login gate', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel('Email').fill(DEMO_EMAIL);
    await page.getByLabel('Password').fill(DEMO_PASSWORD);
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL('/portfolio');

    // Session lives in localStorage; clearing it simulates sign-out / expiry.
    await page.evaluate(() => localStorage.removeItem('honba-session'));
    await page.reload();
    await expect(page).toHaveURL(/.*\/login/);
  });
});
