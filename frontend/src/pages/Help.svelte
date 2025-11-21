<script lang="ts">
</script>

<div class="help-page">
  <h1>Help & FAQ</h1>

  <section>
    <h2>Getting Started</h2>
    <div class="faq-item">
      <h3>How do I create an account?</h3>
      <p>
        Click on the "Register" button and fill in your email, username, and password.
        Once registered, you'll be automatically logged in and can start tracking your poker sessions.
      </p>
    </div>

    <div class="faq-item">
      <h3>How do I add a poker session?</h3>
      <p>
        From your dashboard, click the "Add Session" button. Fill in the details including:
      </p>
      <ul>
        <li><strong>Date:</strong> When you played</li>
        <li><strong>Duration:</strong> How long you played (in minutes)</li>
        <li><strong>Buy-in:</strong> Your initial buy-in amount</li>
        <li><strong>Rebuy:</strong> Any additional money you added (optional)</li>
        <li><strong>Cash Out:</strong> How much you left with</li>
        <li><strong>Notes:</strong> Any additional information about the session</li>
      </ul>
    </div>
  </section>

  <section>
    <h2>Features</h2>
    <div class="faq-item">
      <h3>Understanding Your Stats</h3>
      <p>The dashboard displays several key statistics:</p>
      <ul>
        <li><strong>Total Profit/Loss:</strong> Your overall winnings or losses across all sessions</li>
        <li><strong>Total Sessions:</strong> The number of poker sessions you've recorded</li>
        <li><strong>Total Hours:</strong> Combined duration of all your sessions</li>
        <li><strong>Hourly Rate:</strong> Your profit divided by total hours played</li>
      </ul>
    </div>

    <div class="faq-item">
      <h3>Using the Bankroll Chart</h3>
      <p>
        The chart shows your cumulative profit/loss over time. Use the time range buttons
        (Week, Month, Quarter, Year, All Time) to filter the data and see trends in your performance.
      </p>
    </div>

    <div class="faq-item">
      <h3>Editing and Deleting Sessions</h3>
      <p>
        In the sessions table, each row has Edit and Delete buttons. Click Edit to modify
        session details, or Delete to permanently remove a session from your records.
      </p>
    </div>
  </section>

  <section>
    <h2>Account Management</h2>
    <div class="faq-item">
      <h3>How do I change my password?</h3>
      <p>
        Navigate to Settings from the main menu, then use the "Change Password" form
        to update your password. You'll need to enter your current password for security.
      </p>
    </div>

    <div class="faq-item">
      <h3>What about my privacy?</h3>
      <p>
        We take your privacy seriously. Check our <a href="#/privacy">Privacy Policy</a> for
        detailed information about how we handle your data.
      </p>
    </div>

    <div class="faq-item">
      <h3>Light and Dark Mode</h3>
      <p>
        Use the theme toggle in the navigation bar to switch between light and dark modes.
        Your preference will be saved for future visits.
      </p>
    </div>
  </section>

  <section>
    <h2>Tips for Tracking</h2>
    <div class="faq-item">
      <h3>Best Practices</h3>
      <ul>
        <li>Record sessions immediately after playing for accuracy</li>
        <li>Use the notes field to track game type, location, or specific hands</li>
        <li>Review your stats regularly to identify trends in your play</li>
        <li>Be honest with your records - accurate data leads to better insights</li>
      </ul>
    </div>
  </section>

  <section>
    <h2>Need More Help?</h2>
    <p>
      If you have questions not covered here, please contact support or check our
      <a href="#/privacy">Privacy Policy</a> for more information about data handling.
    </p>
  </section>
</div>

<style>
  .help-page {
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
    color: var(--color-text);
    flex: 1;
  }

  h1 {
    margin-bottom: 2rem;
    color: var(--color-text);
  }

  section {
    margin-bottom: 3rem;
  }

  h2 {
    color: var(--color-text);
    margin-bottom: 1.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 2px solid var(--color-border);
  }

  .faq-item {
    background: var(--color-bg-secondary);
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  h3 {
    color: var(--color-text);
    margin-top: 0;
    margin-bottom: 1rem;
  }

  p {
    line-height: 1.6;
    margin: 0.5rem 0;
    color: var(--color-text);
  }

  ul {
    line-height: 1.8;
    color: var(--color-text);
  }

  a {
    color: var(--color-primary);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  @media (max-width: 768px) {
    .help-page {
      padding: 1rem;
    }

    .faq-item {
      padding: 1rem;
    }
  }
</style>
