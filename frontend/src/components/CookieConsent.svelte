<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';

  let showBanner = false;
  let isAuthenticated = false;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
    if (state.user && !state.user.cookie_consent) {
      showBanner = true;
    } else {
      showBanner = false;
    }
  });

  onMount(() => {
    if (!isAuthenticated && !localStorage.getItem('cookie_consent_dismissed')) {
      showBanner = true;
    }
  });

  async function acceptCookies() {
    if (isAuthenticated) {
      const response = await api.auth.updateCookieConsent(true);
      if (response.data) {
        authStore.updateUser(response.data);
      }
    } else {
      localStorage.setItem('cookie_consent_dismissed', 'true');
    }
    showBanner = false;
  }

  function dismissBanner() {
    if (!isAuthenticated) {
      localStorage.setItem('cookie_consent_dismissed', 'true');
    }
    showBanner = false;
  }
</script>

{#if showBanner}
  <div class="cookie-banner">
    <div class="cookie-content">
      <p>
        We use cookies to enhance your experience, analyze site traffic, and for marketing purposes.
        By clicking "Accept", you consent to our use of cookies.
        Read our <a href="#/privacy">Privacy Policy</a> for more information.
      </p>
      <div class="cookie-actions">
        <button on:click={acceptCookies} class="btn-accept">Accept</button>
        <button on:click={dismissBanner} class="btn-decline">Decline</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .cookie-banner {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    padding: 1.5rem;
    box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
    z-index: 1000;
  }

  .cookie-content {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 2rem;
  }

  .cookie-content p {
    margin: 0;
    flex: 1;
    color: var(--color-text);
  }

  .cookie-content a {
    color: var(--color-primary);
    text-decoration: underline;
  }

  .cookie-actions {
    display: flex;
    gap: 1rem;
  }

  button {
    padding: 0.5rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-accept {
    background-color: var(--color-primary);
    color: white;
  }

  .btn-accept:hover {
    background-color: var(--color-primary-dark);
  }

  .btn-decline {
    background-color: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .btn-decline:hover {
    background-color: var(--color-bg-hover);
  }

  @media (max-width: 768px) {
    .cookie-content {
      flex-direction: column;
      gap: 1rem;
    }

    .cookie-actions {
      width: 100%;
    }

    button {
      flex: 1;
    }
  }
</style>
