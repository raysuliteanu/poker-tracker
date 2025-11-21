<script lang="ts">
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleSubmit() {
    error = '';
    loading = true;

    const response = await api.auth.login(email, password);

    loading = false;

    if (response.error) {
      error = response.error;
    } else if (response.data) {
      authStore.login(response.data.token, response.data.user);
      // Force full route re-evaluation after auth state change
      window.location.hash = '#/';
      window.location.reload();
    }
  }
</script>

<div class="login-container">
  <div class="login-card">
    <h1>Login</h1>
    <p class="subtitle">Welcome back to Poker Bankroll Tracker</p>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <form on:submit|preventDefault={handleSubmit}>
      <div class="form-group">
        <label for="email">Email</label>
        <input
          id="email"
          type="email"
          bind:value={email}
          required
          placeholder="your@email.com"
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          required
          placeholder="Enter your password"
        />
      </div>

      <button type="submit" disabled={loading} class="btn-primary">
        {loading ? 'Logging in...' : 'Login'}
      </button>
    </form>

    <p class="register-link">
      Don't have an account? <a href="#/register">Register here</a>
    </p>
  </div>
</div>

<style>
  .login-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: var(--color-bg);
    width: 100%;
  }

  .login-card {
    background: var(--color-bg-secondary);
    padding: 2.5rem;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    width: 100%;
    max-width: 400px;
  }

  h1 {
    margin: 0 0 0.5rem 0;
    color: var(--color-text);
    text-align: center;
  }

  .subtitle {
    text-align: center;
    color: var(--color-text-secondary);
    margin: 0 0 2rem 0;
  }

  .error {
    background-color: #fee;
    color: #c33;
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--color-text);
    font-weight: 500;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 1rem;
    background: var(--color-bg);
    color: var(--color-text);
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .btn-primary {
    width: 100%;
    padding: 0.75rem;
    background-color: var(--color-primary);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: var(--color-primary-dark);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .register-link {
    text-align: center;
    margin-top: 1.5rem;
    color: var(--color-text-secondary);
  }

  .register-link a {
    color: var(--color-primary);
    text-decoration: none;
    font-weight: 500;
  }

  .register-link a:hover {
    text-decoration: underline;
  }
</style>
