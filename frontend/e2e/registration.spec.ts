import { test, expect } from '@playwright/test';

test.describe('Registration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/register');
  });

  test('displays registration form with all fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Create Account' })).toBeVisible();
    await expect(page.getByLabel('Email')).toBeVisible();
    await expect(page.getByLabel('Username')).toBeVisible();
    await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
    await expect(page.getByLabel('Confirm Password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Register' })).toBeVisible();
  });

  test('shows link to login page', async ({ page }) => {
    const loginLink = page.getByRole('link', { name: 'Login here' });
    await expect(loginLink).toBeVisible();
    await loginLink.click();
    await expect(page).toHaveURL(/#\/login/);
  });

  test('shows error when passwords do not match', async ({ page }) => {
    await page.getByLabel('Email').fill('test@example.com');
    await page.getByLabel('Username').fill('testuser');
    await page.getByLabel('Password', { exact: true }).fill('password123');
    await page.getByLabel('Confirm Password').fill('differentpassword');
    await page.getByRole('button', { name: 'Register' }).click();

    await expect(page.getByText('Passwords do not match')).toBeVisible();
  });

  test('shows error when password is too short', async ({ page }) => {
    await page.getByLabel('Email').fill('test@example.com');
    await page.getByLabel('Username').fill('testuser');
    await page.getByLabel('Password', { exact: true }).fill('short');
    await page.getByLabel('Confirm Password').fill('short');
    await page.getByRole('button', { name: 'Register' }).click();

    await expect(page.getByText('Password must be at least 8 characters')).toBeVisible();
  });

  test('successfully registers a new user', async ({ page }) => {
    // Generate unique user to avoid conflicts on reruns
    const timestamp = Date.now();
    const email = `e2e-test-${timestamp}@example.com`;
    const username = `e2euser${timestamp}`;

    await page.getByLabel('Email').fill(email);
    await page.getByLabel('Username').fill(username);
    await page.getByLabel('Password', { exact: true }).fill('securepassword123');
    await page.getByLabel('Confirm Password').fill('securepassword123');
    await page.getByRole('button', { name: 'Register' }).click();

    // Should show success message
    await expect(page.getByText(/Registration successful/)).toBeVisible();

    // Should redirect to dashboard after registration
    await expect(page).toHaveURL(/#\//, { timeout: 5000 });
  });

  test('shows error when registering with existing email', async ({ page }) => {
    // First registration
    const timestamp = Date.now();
    const email = `duplicate-${timestamp}@example.com`;
    const username1 = `user1-${timestamp}`;

    await page.getByLabel('Email').fill(email);
    await page.getByLabel('Username').fill(username1);
    await page.getByLabel('Password', { exact: true }).fill('securepassword123');
    await page.getByLabel('Confirm Password').fill('securepassword123');
    await page.getByRole('button', { name: 'Register' }).click();

    await expect(page.getByText(/Registration successful/)).toBeVisible();

    // Wait for redirect and go back to register
    await page.waitForURL(/#\//);
    await page.goto('/#/register');

    // Try to register with same email
    const username2 = `user2-${timestamp}`;
    await page.getByLabel('Email').fill(email);
    await page.getByLabel('Username').fill(username2);
    await page.getByLabel('Password', { exact: true }).fill('securepassword123');
    await page.getByLabel('Confirm Password').fill('securepassword123');
    await page.getByRole('button', { name: 'Register' }).click();

    // Should show error about existing email
    await expect(page.getByText(/email.*already|already.*registered/i)).toBeVisible({ timeout: 5000 });
  });
});
