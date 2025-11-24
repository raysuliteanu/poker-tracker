import { test, expect } from '@playwright/test';

/**
 * Helper function to log in a user for dashboard tests.
 * Creates a new user with a timestamp to avoid conflicts.
 */
async function loginUser(page: any) {
  const timestamp = Date.now();
  const email = `dashboard-test-${timestamp}@example.com`;
  const username = `dashuser${timestamp}`;
  const password = 'testpassword123';

  // Register user
  await page.goto('/#/register');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Username').fill(username);
  await page.getByLabel('Password', { exact: true }).fill(password);
  await page.getByLabel('Confirm Password').fill(password);
  await page.getByRole('button', { name: 'Register' }).click();

  // Wait for redirect to dashboard
  await page.waitForURL(/#\//);
  await expect(page.getByRole('heading', { name: 'Poker Bankroll Tracker' })).toBeVisible();

  // Dismiss cookie consent banner if it appears
  const acceptCookiesButton = page.getByRole('button', { name: 'Accept' });
  if (await acceptCookiesButton.isVisible({ timeout: 2000 }).catch(() => false)) {
    await acceptCookiesButton.click();
    await page.waitForTimeout(500); // Wait for banner to disappear
  }

  return { email, username, password };
}

/**
 * Helper function to add a poker session via the UI
 */
async function addSession(
  page: any,
  sessionData: {
    date: string;
    duration: number;
    buyIn: number;
    rebuy: number;
    cashOut: number;
    notes?: string;
  }
) {
  await page.getByRole('button', { name: 'Add Session' }).click();
  await expect(page.getByRole('heading', { name: 'Add New Session' })).toBeVisible();

  await page.getByLabel('Session Date').fill(sessionData.date);
  await page.getByLabel('Duration (minutes)').fill(sessionData.duration.toString());
  await page.getByLabel('Buy-in ($)').fill(sessionData.buyIn.toString());
  await page.getByLabel('Rebuy ($)').fill(sessionData.rebuy.toString());
  await page.getByLabel('Cash Out ($)').fill(sessionData.cashOut.toString());

  if (sessionData.notes) {
    await page.getByLabel('Notes (optional)').fill(sessionData.notes);
  }

  await page.getByRole('button', { name: 'Save Session' }).click();

  // Wait for modal to close
  await expect(page.getByRole('heading', { name: 'Add New Session' })).not.toBeVisible();
}

test.describe('Dashboard - Poker Sessions', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('displays empty dashboard with zero stats', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Poker Bankroll Tracker' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add Session' })).toBeVisible();

    // Check stats cards show zeros
    await expect(page.getByText('Total Profit/Loss')).toBeVisible();
    await expect(page.locator('.stat-value').filter({ hasText: '$0.00' }).first()).toBeVisible();
    await expect(page.getByText('Total Sessions')).toBeVisible();
    await expect(page.locator('.stat-value').filter({ hasText: /^0$/ })).toBeVisible();
    await expect(page.getByText('Total Hours')).toBeVisible();
    await expect(page.getByText('Hourly Rate')).toBeVisible();
  });

  test('opens and closes add session form', async ({ page }) => {
    await page.getByRole('button', { name: 'Add Session' }).click();

    // Form should be visible
    await expect(page.getByRole('heading', { name: 'Add New Session' })).toBeVisible();
    await expect(page.getByLabel('Session Date')).toBeVisible();
    await expect(page.getByLabel('Duration (minutes)')).toBeVisible();
    await expect(page.getByLabel('Buy-in ($)')).toBeVisible();
    await expect(page.getByLabel('Rebuy ($)')).toBeVisible();
    await expect(page.getByLabel('Cash Out ($)')).toBeVisible();
    await expect(page.getByLabel('Notes (optional)')).toBeVisible();

    // Close form with Cancel button
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByRole('heading', { name: 'Add New Session' })).not.toBeVisible();
  });

  test('closes add session form by clicking overlay', async ({ page }) => {
    await page.getByRole('button', { name: 'Add Session' }).click();
    await expect(page.getByRole('heading', { name: 'Add New Session' })).toBeVisible();

    // Click on the modal overlay (not the modal itself)
    await page.locator('.modal-overlay').click({ position: { x: 10, y: 10 } });
    await expect(page.getByRole('heading', { name: 'Add New Session' })).not.toBeVisible();
  });

  test('successfully adds a winning poker session', async ({ page }) => {
    const sessionDate = '2024-01-15';
    const duration = 180; // 3 hours
    const buyIn = 100;
    const rebuy = 50;
    const cashOut = 300;
    const notes = 'Great session, played well';
    const expectedProfit = cashOut - (buyIn + rebuy); // $150

    await addSession(page, {
      date: sessionDate,
      duration,
      buyIn,
      rebuy,
      cashOut,
      notes,
    });

    // Verify session appears in the table
    await expect(page.locator('table tbody')).toContainText('1/15/2024'); // Date formatted by browser
    await expect(page.locator('table tbody')).toContainText('$150.00'); // Profit

    // Verify notes are accessible via the view notes button
    await expect(page.locator('button[aria-label="View notes"]')).toBeVisible();
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByText(notes)).toBeVisible();
    // Close the modal
    await page.getByRole('button', { name: 'Close' }).click();

    // Verify stats updated correctly
    await expect(page.locator('.stat-value.profit')).toContainText('$150.00'); // Total profit
    await expect(page.locator('.stat-value').filter({ hasText: /^1$/ })).toBeVisible(); // Total sessions
    await expect(page.locator('.stat-value').filter({ hasText: '3.0' })).toBeVisible(); // Total hours
    await expect(page.locator('.stat-value').filter({ hasText: '$50.00/hr' })).toBeVisible(); // Hourly rate
  });

  test('successfully adds a losing poker session', async ({ page }) => {
    const sessionDate = '2024-01-20';
    const duration = 120; // 2 hours
    const buyIn = 200;
    const rebuy = 100;
    const cashOut = 150;
    const expectedProfit = cashOut - (buyIn + rebuy); // -$150

    await addSession(page, {
      date: sessionDate,
      duration,
      buyIn,
      rebuy,
      cashOut,
    });

    // Verify session appears with negative profit in the table
    await expect(page.locator('table tbody')).toContainText('$-150.00');

    // Verify stats show loss (should be red)
    await expect(page.locator('.stat-value.loss')).toContainText('$-150.00');
  });

  test('adds multiple sessions and verifies stats calculations', async ({ page }) => {
    // Session 1: Win $100 in 2 hours
    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
    });

    // Session 2: Lose $50 in 1 hour
    await addSession(page, {
      date: '2024-01-11',
      duration: 60,
      buyIn: 100,
      rebuy: 0,
      cashOut: 50,
    });

    // Session 3: Win $200 in 3 hours
    await addSession(page, {
      date: '2024-01-12',
      duration: 180,
      buyIn: 100,
      rebuy: 50,
      cashOut: 350,
    });

    // Total: +$100 -$50 +$200 = +$250 over 6 hours = $41.67/hr
    await expect(page.locator('.stat-value.profit')).toContainText('$250.00'); // Total profit
    await expect(page.locator('.stat-value').filter({ hasText: /^3$/ })).toBeVisible(); // Total sessions
    await expect(page.locator('.stat-value').filter({ hasText: '6.0' })).toBeVisible(); // Total hours
    await expect(page.locator('.stat-value').filter({ hasText: '$41.67/hr' })).toBeVisible(); // Hourly rate
  });

  test('validates required fields when adding session', async ({ page }) => {
    await page.getByRole('button', { name: 'Add Session' }).click();

    // Try to submit without filling required fields
    await page.getByRole('button', { name: 'Save Session' }).click();

    // Form should not close (validation should prevent submission)
    await expect(page.getByRole('heading', { name: 'Add New Session' })).toBeVisible();
  });

  test('adds session with zero rebuy', async ({ page }) => {
    await addSession(page, {
      date: '2024-01-15',
      duration: 60,
      buyIn: 100,
      rebuy: 0,
      cashOut: 150,
    });

    await expect(page.locator('table tbody')).toContainText('$50.00'); // Profit
  });
});

test.describe('Dashboard - Edit Sessions', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);

    // Add an initial session to edit
    await addSession(page, {
      date: '2024-01-15',
      duration: 120,
      buyIn: 100,
      rebuy: 50,
      cashOut: 200,
      notes: 'Original notes',
    });
  });

  test('opens edit form with pre-filled data', async ({ page }) => {
    // Click edit button (assuming it's an edit icon or button in the table)
    await page.locator('button[aria-label="Edit session"]').first().click();

    // Verify form opens with "Edit Session" title
    await expect(page.getByRole('heading', { name: 'Edit Session' })).toBeVisible();

    // Verify fields are pre-filled with existing data
    await expect(page.getByLabel('Session Date')).toHaveValue('2024-01-15');
    await expect(page.getByLabel('Duration (minutes)')).toHaveValue('120');
    await expect(page.getByLabel('Buy-in ($)')).toHaveValue('100');
    await expect(page.getByLabel('Rebuy ($)')).toHaveValue('50');
    await expect(page.getByLabel('Cash Out ($)')).toHaveValue('200');
    await expect(page.getByLabel('Notes (optional)')).toHaveValue('Original notes');
  });

  test('successfully edits session and updates stats', async ({ page }) => {
    // Click edit button
    await page.locator('button[aria-label="Edit session"]').first().click();

    // Modify the session
    await page.getByLabel('Duration (minutes)').fill('180'); // Change from 120 to 180
    await page.getByLabel('Cash Out ($)').fill('300'); // Change from 200 to 300
    await page.getByLabel('Notes (optional)').fill('Updated notes after edit');

    await page.getByRole('button', { name: 'Save Session' }).click();

    // Wait for modal to close
    await expect(page.getByRole('heading', { name: 'Edit Session' })).not.toBeVisible();

    // Verify updated data in table
    await expect(page.locator('table tbody')).toContainText('$150.00'); // New profit: 300 - 150 = 150

    // Verify updated notes via view notes button
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByText('Updated notes after edit')).toBeVisible();
    await page.getByRole('button', { name: 'Close' }).click();

    // Verify stats updated correctly (3 hours now instead of 2)
    await expect(page.getByText('3.0')).toBeVisible(); // Total hours
    await expect(page.getByText('$50.00/hr')).toBeVisible(); // Updated hourly rate
  });

  test('cancels edit without saving changes', async ({ page }) => {
    // Click edit button
    await page.locator('button[aria-label="Edit session"]').first().click();

    // Make changes
    await page.getByLabel('Cash Out ($)').fill('500');
    await page.getByLabel('Notes (optional)').fill('This should not be saved');

    // Cancel the edit
    await page.getByRole('button', { name: 'Cancel' }).click();

    // Verify modal closed
    await expect(page.getByRole('heading', { name: 'Edit Session' })).not.toBeVisible();

    // Verify original data still in table (profit still $50)
    await expect(page.locator('table tbody')).toContainText('$50.00');

    // Verify original notes still there via view notes button
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByText('Original notes')).toBeVisible();
    await expect(page.getByText('This should not be saved')).not.toBeVisible();
    await page.getByRole('button', { name: 'Close' }).click();
  });

  test('edits session to change from win to loss', async ({ page }) => {
    // Original: buy-in $100, rebuy $50, cash out $200 = +$50 profit

    await page.locator('button[aria-label="Edit session"]').first().click();

    // Change cash out to make it a loss
    await page.getByLabel('Cash Out ($)').fill('100'); // Now -$50 loss

    await page.getByRole('button', { name: 'Save Session' }).click();

    // Wait for update
    await expect(page.getByRole('heading', { name: 'Edit Session' })).not.toBeVisible();

    // Verify profit changed to loss in the table
    await expect(page.locator('table tbody')).toContainText('$-50.00');

    // Verify total profit/loss stat shows loss
    await expect(page.locator('.stat-value.loss')).toContainText('$-50.00');
  });
});

test.describe('Dashboard - Delete Sessions', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);

    // Add a session to delete
    await addSession(page, {
      date: '2024-01-15',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Session to delete',
    });
  });

  test('deletes session with confirmation', async ({ page }) => {
    // Set up dialog handler to accept the confirmation
    page.on('dialog', async dialog => {
      expect(dialog.type()).toBe('confirm');
      expect(dialog.message()).toContain('Are you sure you want to delete this session?');
      await dialog.accept();
    });

    // Click delete button
    await page.locator('button[aria-label="Delete session"]').first().click();

    // Wait a moment for the deletion to process
    await page.waitForTimeout(500);

    // Verify session is gone - no view notes button should be present
    await expect(page.locator('button[aria-label="View notes"]')).not.toBeVisible();

    // Verify stats reset to zero
    await expect(page.locator('.stat-value').filter({ hasText: '$0.00' }).first()).toBeVisible();
    await expect(page.locator('.stat-value').filter({ hasText: /^0$/ })).toBeVisible(); // Total sessions
  });

  test('cancels deletion when user dismisses confirmation', async ({ page }) => {
    // Set up dialog handler to dismiss the confirmation
    page.on('dialog', async dialog => {
      expect(dialog.type()).toBe('confirm');
      await dialog.dismiss();
    });

    // Click delete button
    await page.locator('button[aria-label="Delete session"]').first().click();

    // Wait a moment
    await page.waitForTimeout(500);

    // Verify session is still there - view notes button and profit still showing
    await expect(page.locator('button[aria-label="View notes"]')).toBeVisible();
    await expect(page.locator('table tbody')).toContainText('$100.00'); // Profit still showing
  });
});

test.describe('Dashboard - Notes Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('shows view notes button only for sessions with notes', async ({ page }) => {
    // Add session with notes
    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Session with notes',
    });

    // Add session without notes
    await addSession(page, {
      date: '2024-01-11',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
    });

    // Should have exactly 1 view notes button
    const viewNotesButtons = page.locator('button[aria-label="View notes"]');
    await expect(viewNotesButtons).toHaveCount(1);
  });

  test('opens notes modal when view notes button is clicked', async ({ page }) => {
    const notes = 'This is a test note with multiple lines\nLine 2\nLine 3';

    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes,
    });

    // Click view notes button
    await page.locator('button[aria-label="View notes"]').first().click();

    // Modal should be visible with the correct content
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.getByText(/Session Notes -/)).toBeVisible();
    await expect(page.getByText(notes)).toBeVisible();
  });

  test('closes notes modal when close button is clicked', async ({ page }) => {
    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Test notes',
    });

    // Open modal
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Close modal with close button
    await page.getByRole('button', { name: 'Close' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('closes notes modal when clicking overlay', async ({ page }) => {
    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Test notes',
    });

    // Open modal
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Close modal by clicking overlay
    await page.locator('.modal-overlay').click({ position: { x: 10, y: 10 } });
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('closes notes modal with Escape key', async ({ page }) => {
    await addSession(page, {
      date: '2024-01-10',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Test notes',
    });

    // Open modal
    await page.locator('button[aria-label="View notes"]').first().click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Close modal with Escape key
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('displays session date in modal header', async ({ page }) => {
    await addSession(page, {
      date: '2024-01-15',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
      notes: 'Test notes',
    });

    // Open modal
    await page.locator('button[aria-label="View notes"]').first().click();

    // Verify modal header contains the session date
    await expect(page.getByText(/Session Notes - .*1\/15\/2024/)).toBeVisible();
  });
});

test.describe('Dashboard - Charts Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('navigates to Charts page from navbar', async ({ page }) => {
    // Should be on dashboard initially
    await expect(page.getByRole('heading', { name: 'Poker Bankroll Tracker' })).toBeVisible();

    // Click Charts link in navbar
    await page.getByRole('link', { name: 'Charts' }).click();

    // Should navigate to Charts page
    await page.waitForURL(/#\/charts/);
    await expect(page.getByRole('heading', { name: 'Charts & Analytics' })).toBeVisible();
  });

  test('charts page displays heading', async ({ page }) => {
    // Navigate to Charts
    await page.getByRole('link', { name: 'Charts' }).click();
    await page.waitForURL(/#\/charts/);

    // Verify Charts page heading is displayed
    await expect(page.getByRole('heading', { name: 'Charts & Analytics' })).toBeVisible();
  });

  test('charts page shows empty state when no sessions', async ({ page }) => {
    // Navigate to Charts immediately (no sessions added)
    await page.getByRole('link', { name: 'Charts' }).click();
    await page.waitForURL(/#\/charts/);

    // Should show empty state
    await expect(page.getByText('No poker sessions recorded yet.')).toBeVisible();
    await expect(page.getByText('Add sessions from the Dashboard to see charts!')).toBeVisible();
  });

  test('charts page displays bankroll chart when sessions exist', async ({ page }) => {
    // Add sessions
    await addSession(page, {
      date: '2024-01-15',
      duration: 120,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
    });

    // Navigate to Charts
    await page.getByRole('link', { name: 'Charts' }).click();
    await page.waitForURL(/#\/charts/);

    // Verify chart is displayed (look for the chart title)
    await expect(page.getByText('Bankroll Over Time')).toBeVisible();
  });
});

test.describe('Dashboard - Stats Display', () => {
  test.beforeEach(async ({ page }) => {
    await loginUser(page);
  });

  test('displays correct stats for multiple sessions', async ({ page }) => {
    // Add 3 sessions with known values
    await addSession(page, {
      date: '2024-01-10',
      duration: 120, // 2 hours
      buyIn: 100,
      rebuy: 0,
      cashOut: 200, // +$100
    });

    await addSession(page, {
      date: '2024-01-11',
      duration: 180, // 3 hours
      buyIn: 200,
      rebuy: 100,
      cashOut: 150, // -$150
    });

    await addSession(page, {
      date: '2024-01-12',
      duration: 240, // 4 hours
      buyIn: 100,
      rebuy: 0,
      cashOut: 300, // +$200
    });

    // Total: +$100 -$150 +$200 = +$150
    // Hours: 2 + 3 + 4 = 9 hours
    // Hourly: $150 / 9 = $16.67/hr
    // Sessions: 3

    await expect(page.locator('.stat-value.profit')).toContainText('$150.00');
    await expect(page.locator('.stat-value').filter({ hasText: /^3$/ })).toBeVisible();
    await expect(page.locator('.stat-value').filter({ hasText: '9.0' })).toBeVisible();
    await expect(page.locator('.stat-value').filter({ hasText: '$16.67/hr' })).toBeVisible();
  });

  test('displays profit in green and loss in red', async ({ page }) => {
    // Add winning session
    await addSession(page, {
      date: '2024-01-10',
      duration: 60,
      buyIn: 100,
      rebuy: 0,
      cashOut: 200,
    });

    // Verify profit is displayed with green color class
    await expect(page.locator('.stat-value.profit')).toBeVisible();

    // Now add a larger losing session to make total negative
    await addSession(page, {
      date: '2024-01-11',
      duration: 60,
      buyIn: 200,
      rebuy: 100,
      cashOut: 50,
    });

    // Total should now be -$150, displayed in red
    await expect(page.locator('.stat-value.loss')).toContainText('$-150.00');
  });

  test('handles zero hours edge case for hourly rate', async ({ page }) => {
    // This shouldn't normally happen, but test the edge case
    // Dashboard should show $0.00/hr when no sessions exist
    await expect(page.getByText('$0.00/hr')).toBeVisible();
  });
});
