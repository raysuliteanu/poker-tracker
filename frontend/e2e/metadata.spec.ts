import { test, expect } from '@playwright/test';

test.describe('Page Metadata', () => {
  test('has correct page title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle('Poker Bankroll Tracker');
  });

  test('has favicon set correctly', async ({ page }) => {
    await page.goto('/');

    // Get the favicon link element
    const faviconLink = page.locator('link[rel="icon"]');
    await expect(faviconLink).toHaveAttribute('href', '/favicon.ico');
    await expect(faviconLink).toHaveAttribute('type', 'image/x-icon');

    // Verify the favicon file exists and loads successfully
    const response = await page.goto('/favicon.ico');
    expect(response?.status()).toBe(200);
    expect(response?.headers()['content-type']).toContain('image');
  });
});
