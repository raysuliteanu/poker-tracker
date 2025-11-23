<script lang="ts">
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';

  let email = '';
  let username = '';
  let password = '';
  let confirmPassword = '';
  let error = '';
  let success = '';
  let loading = false;

  async function handleSubmit() {
    error = '';
    success = '';

    if (password !== confirmPassword) {
      error = 'Passwords do not match';
      return;
    }

    if (password.length < 8) {
      error = 'Password must be at least 8 characters';
      return;
    }

    loading = true;

    const response = await api.auth.register(email, username, password);

    loading = false;

    if (response.error) {
      error = response.error;
    } else if (response.data) {
      authStore.login(response.data.token, response.data.user);
      success = 'Registration successful! Redirecting to dashboard...';
      // Use location.hash to force full route re-evaluation after auth state change
      setTimeout(() => {
        window.location.hash = '#/';
        window.location.reload();
      }, 1000);
    }
  }
</script>

<div class="register-container">
  <div class="register-card">
    <h1>Create Account</h1>
    <p class="subtitle">Start tracking your poker bankroll today</p>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    {#if success}
      <div class="success">{success}</div>
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
        <label for="username">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          required
          minlength="3"
          maxlength="100"
          placeholder="Choose a username"
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          required
          placeholder="At least 8 characters"
        />
      </div>

      <div class="form-group">
        <label for="confirmPassword">Confirm Password</label>
        <input
          id="confirmPassword"
          type="password"
          bind:value={confirmPassword}
          required
          placeholder="Re-enter your password"
        />
      </div>

      <button type="submit" disabled={loading} class="btn-primary">
        {loading ? 'Creating account...' : 'Register'}
      </button>
    </form>

    <p class="login-link">
      Already have an account? <a href="#/login">Login here</a>
    </p>
  </div>
</div>

<style>
  .register-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: var(--color-bg);
    width: 100%;
  }

  .register-card {
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

  .success {
    background-color: #efe;
    color: #2a2;
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

  .login-link {
    text-align: center;
    margin-top: 1.5rem;
    color: var(--color-text-secondary);
  }

  .login-link a {
    color: var(--color-primary);
    text-decoration: none;
    font-weight: 500;
  }

  .login-link a:hover {
    text-decoration: underline;
  }
</style>
