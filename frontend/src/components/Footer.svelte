<script lang="ts">
  import { link } from 'svelte-spa-router';
</script>

<footer class="footer">
  <div class="footer-container">
    <div class="footer-links">
      <a href="#/help" use:link>Help</a>
      <span class="separator">|</span>
      <a href="#/privacy" use:link>Privacy</a>
    </div>
    <p class="copyright">Poker Bankroll Tracker</p>
  </div>
</footer>

<style>
  .footer {
    background: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    padding: 0.5rem 0;
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 100;
  }

  .footer-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .footer-links {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .footer-links a {
    color: var(--color-text-secondary);
    text-decoration: none;
    font-size: 0.875rem;
    transition: color 0.2s;
  }

  .footer-links a:hover {
    color: var(--color-primary);
  }

  .separator {
    color: var(--color-text-secondary);
  }

  .copyright {
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    margin: 0;
  }

  @media (max-width: 768px) {
    .footer-container {
      flex-direction: column;
      gap: 0.5rem;
      text-align: center;
    }
  }
</style>
