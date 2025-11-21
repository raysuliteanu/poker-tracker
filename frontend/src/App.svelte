<script lang="ts">
  import { onMount } from 'svelte';
  import Router, { push, location } from 'svelte-spa-router';
  import { authStore } from './stores/auth';
  import { themeStore } from './stores/theme';
  import { api } from './lib/api';

  import Navigation from './components/Navigation.svelte';
  import Footer from './components/Footer.svelte';
  import CookieConsent from './components/CookieConsent.svelte';

  import Landing from './pages/Landing.svelte';
  import Login from './pages/Login.svelte';
  import Register from './pages/Register.svelte';
  import Dashboard from './pages/Dashboard.svelte';
  import Settings from './pages/Settings.svelte';
  import Help from './pages/Help.svelte';
  import Privacy from './pages/Privacy.svelte';

  // Public routes available without authentication
  const publicRoutes = {
    '/': Landing,
    '/login': Login,
    '/register': Register,
    '/help': Help,
    '/privacy': Privacy,
  };

  // Protected routes requiring authentication
  const protectedRoutes = {
    '/': Dashboard,
    '/settings': Settings,
    '/help': Help,
    '/privacy': Privacy,
  };

  // Reactive routes based on auth state
  $: routes = $authStore.isAuthenticated ? protectedRoutes : publicRoutes;

  let isReady = false;

  onMount(async () => {
    // Initialize theme
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
    themeStore.set(savedTheme || 'light');

    // Check if user is logged in - validate token silently
    if ($authStore.isAuthenticated) {
      const response = await api.auth.getMe();
      if (response.data) {
        authStore.updateUser(response.data);
      } else {
        // Token invalid - logout silently, no error shown
        authStore.logout();
      }
    }

    isReady = true;
  });

  function conditionsFailed(event: any) {
    // Redirect to landing if trying to access protected route while not authenticated
    if (!$authStore.isAuthenticated) {
      push('/');
    }
  }
</script>

{#if isReady}
  <div class="app">
    <Navigation />
    <main>
      <Router {routes} on:conditionsFailed={conditionsFailed} />
    </main>
    <Footer />
    <CookieConsent />
  </div>
{:else}
  <div class="loading-screen">
    <div class="spinner"></div>
  </div>
{/if}

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(html) {
    overflow-y: scroll;
    overflow-x: hidden;
    width: 100%;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Helvetica Neue', sans-serif;
    background: var(--color-bg);
    color: var(--color-text);
    transition: background-color 0.3s, color 0.3s;
    min-width: 320px;
    overflow-x: hidden;
    width: 100vw;
  }

  :global([data-theme='light']) {
    --color-bg: #f5f5f5;
    --color-bg-secondary: #ffffff;
    --color-bg-hover: #f9f9f9;
    --color-text: #1a1a1a;
    --color-text-secondary: #666666;
    --color-primary: #3b82f6;
    --color-primary-dark: #2563eb;
    --color-border: #e5e5e5;
  }

  :global([data-theme='dark']) {
    --color-bg: #1a1a1a;
    --color-bg-secondary: #2d2d2d;
    --color-bg-hover: #3a3a3a;
    --color-text: #ffffff;
    --color-text-secondary: #a0a0a0;
    --color-primary: #3b82f6;
    --color-primary-dark: #2563eb;
    --color-border: #404040;
  }

  .app {
    min-height: 100vh;
    display: grid;
    grid-template-rows: auto 1fr auto;
    width: 100%;
  }

  main {
    display: block;
    width: 100%;
    padding-top: 80px;
    padding-bottom: 60px;
  }

  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: var(--color-bg);
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
