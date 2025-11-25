<script lang="ts">
  import { push } from 'svelte-spa-router';
</script>

<div class="landing-container">
  <div class="landing-content">
    <div class="hero">
      <h1>Poker Bankroll Tracker</h1>
      <p class="tagline">Track your poker sessions, analyze your performance, and grow your bankroll.</p>
    </div>

    <div class="features">
      <div class="feature">
        <div class="feature-icon">📊</div>
        <h3>Track Sessions</h3>
        <p>Log your buy-ins, cash-outs, and session duration with detailed notes.</p>
      </div>
      <div class="feature">
        <div class="feature-icon">📈</div>
        <h3>Analyze Performance</h3>
        <p>View your profit/loss over time with interactive charts and statistics.</p>
      </div>
      <div class="feature">
        <div class="feature-icon">💰</div>
        <h3>Grow Your Bankroll</h3>
        <p>Make data-driven decisions to improve your game and maximize profits.</p>
      </div>
    </div>

    <div class="cta-buttons">
      <button class="btn-primary" on:click={() => push('/register')}>
        Get Started
      </button>
    </div>

    <p class="login-hint">
      Already have an account? <a href="#/login">Sign in here</a>
    </p>
  </div>
</div>

<style>
  .landing-container {
    flex: 1;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 2rem 2rem 2rem 2rem;
    padding-top: 1rem;
    background: var(--color-bg);
    width: 100%;
  }

  .landing-content {
    max-width: 800px;
    text-align: center;
  }

  .hero {
    margin-bottom: 2rem;
  }

  h1 {
    font-size: 3rem;
    color: var(--color-text);
    margin: 0 0 1rem 0;
  }

  .tagline {
    font-size: 1.25rem;
    color: var(--color-text-secondary);
    margin: 0;
    max-width: 500px;
    margin: 0 auto;
  }

  .features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .feature {
    background: var(--color-bg-secondary);
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .feature-icon {
    font-size: 2.5rem;
    margin-bottom: 0.75rem;
  }

  .feature h3 {
    color: var(--color-text);
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
  }

  .feature p {
    color: var(--color-text-secondary);
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .cta-buttons {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-bottom: 1.5rem;
  }

  .btn-primary {
    padding: 0.875rem 2rem;
    font-size: 1rem;
    font-weight: 600;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    background-color: var(--color-primary);
    color: white;
    border: none;
  }

  .btn-primary:hover {
    background-color: var(--color-primary-dark);
  }

  .login-hint {
    color: var(--color-text-secondary);
    font-size: 0.9rem;
  }

  .login-hint a {
    color: var(--color-primary);
    text-decoration: none;
    font-weight: 500;
  }

  .login-hint a:hover {
    text-decoration: underline;
  }

  @media (max-width: 600px) {
    h1 {
      font-size: 2rem;
    }

    .tagline {
      font-size: 1rem;
    }

    .cta-buttons {
      flex-direction: column;
    }

    .btn-primary {
      width: 100%;
    }
  }
</style>
