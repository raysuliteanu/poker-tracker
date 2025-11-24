<script lang="ts">
  import { link } from 'svelte-spa-router';
  import { authStore } from '../stores/auth';
  import { themeStore } from '../stores/theme';

  let currentTheme: 'light' | 'dark' = 'light';

  themeStore.subscribe((theme) => {
    currentTheme = theme;
  });

  function handleLogout() {
    authStore.logout();
    // Force full route re-evaluation after auth state change
    window.location.hash = '#/';
    window.location.reload();
  }

  function toggleTheme() {
    themeStore.set(currentTheme === 'light' ? 'dark' : 'light');
  }
</script>

{#if $authStore.isAuthenticated}
  <nav class="navbar">
    <div class="nav-container">
      <div class="nav-brand">
        <a href="#/" use:link>Poker Tracker</a>
      </div>

      <div class="nav-links">
        <a href="#/" use:link>Dashboard</a>
        <a href="#/charts" use:link>Charts</a>
        <a href="#/settings" use:link>Settings</a>
        <button on:click={toggleTheme} class="theme-toggle" title="Toggle theme">
          {#if currentTheme === 'light'}
            🌙
          {:else}
            ☀️
          {/if}
        </button>
        <button on:click={handleLogout} class="btn-logout">Logout</button>
      </div>
    </div>
  </nav>
{/if}

<style>
  .navbar {
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    padding: 1rem 0;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
  }

  .nav-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .nav-brand a {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-primary);
    text-decoration: none;
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .nav-links a {
    color: var(--color-text);
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
  }

  .nav-links a:hover {
    color: var(--color-primary);
  }

  .theme-toggle {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
    transition: transform 0.2s;
  }

  .theme-toggle:hover {
    transform: scale(1.1);
  }

  .btn-logout {
    padding: 0.5rem 1rem;
    background-color: var(--color-primary);
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .btn-logout:hover {
    background-color: var(--color-primary-dark);
  }

  @media (max-width: 768px) {
    .nav-container {
      padding: 0 1rem;
    }

    .nav-links {
      gap: 0.75rem;
    }

    .nav-links a {
      font-size: 0.875rem;
    }

    .btn-logout {
      padding: 0.375rem 0.75rem;
      font-size: 0.875rem;
    }
  }
</style>
