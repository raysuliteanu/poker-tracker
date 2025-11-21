<script lang="ts">
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';

  let oldPassword = '';
  let newPassword = '';
  let confirmPassword = '';
  let error = '';
  let success = '';
  let loading = false;

  async function handleChangePassword() {
    error = '';
    success = '';

    if (newPassword !== confirmPassword) {
      error = 'New passwords do not match';
      return;
    }

    if (newPassword.length < 8) {
      error = 'New password must be at least 8 characters';
      return;
    }

    loading = true;

    const response = await api.auth.changePassword(oldPassword, newPassword);

    loading = false;

    if (response.error) {
      error = response.error;
    } else {
      success = 'Password changed successfully!';
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';
    }
  }
</script>

<div class="settings-page">
  <h1>Settings</h1>

  <div class="settings-section">
    <h2>Change Password</h2>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    {#if success}
      <div class="success">{success}</div>
    {/if}

    <form on:submit|preventDefault={handleChangePassword}>
      <div class="form-group">
        <label for="oldPassword">Current Password</label>
        <input
          id="oldPassword"
          type="password"
          bind:value={oldPassword}
          required
          placeholder="Enter current password"
        />
      </div>

      <div class="form-group">
        <label for="newPassword">New Password</label>
        <input
          id="newPassword"
          type="password"
          bind:value={newPassword}
          required
          minlength="8"
          placeholder="At least 8 characters"
        />
      </div>

      <div class="form-group">
        <label for="confirmPassword">Confirm New Password</label>
        <input
          id="confirmPassword"
          type="password"
          bind:value={confirmPassword}
          required
          placeholder="Re-enter new password"
        />
      </div>

      <button type="submit" disabled={loading} class="btn-primary">
        {loading ? 'Changing...' : 'Change Password'}
      </button>
    </form>
  </div>

  <div class="settings-section">
    <h2>Account Information</h2>
    {#if $authStore.user}
      <div class="info-item">
        <span class="label">Email:</span>
        <span class="value">{$authStore.user.email}</span>
      </div>
      <div class="info-item">
        <span class="label">Username:</span>
        <span class="value">{$authStore.user.username}</span>
      </div>
      <div class="info-item">
        <span class="label">Member Since:</span>
        <span class="value">
          {new Date($authStore.user.created_at).toLocaleDateString()}
        </span>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-page {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    flex: 1;
  }

  h1 {
    margin-bottom: 2rem;
    color: var(--color-text);
  }

  .settings-section {
    background: var(--color-bg-secondary);
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
  }

  h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    color: var(--color-text);
    font-size: 1.25rem;
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
    color: #363;
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
    padding: 0.75rem 1.5rem;
    background-color: var(--color-primary);
    color: white;
    border: none;
    border-radius: 4px;
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

  .info-item {
    display: flex;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--color-border);
  }

  .info-item:last-child {
    border-bottom: none;
  }

  .label {
    font-weight: 600;
    color: var(--color-text);
    width: 150px;
  }

  .value {
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .settings-page {
      padding: 1rem;
    }

    .settings-section {
      padding: 1.5rem;
    }
  }
</style>
