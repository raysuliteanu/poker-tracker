<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api, type PokerSession } from '../lib/api';
  import BankrollChart from '../components/BankrollChart.svelte';

  let sessions: PokerSession[] = [];
  let loading = true;
  let error = '';

  onMount(async () => {
    await loadSessions();
  });

  async function loadSessions() {
    loading = true;
    error = '';

    const response = await api.sessions.getAll();

    loading = false;

    if (response.error) {
      error = response.error;
    } else if (response.data) {
      sessions = response.data;
    }
  }

</script>

<div class="charts">
  <div class="header">
    <h1>Charts & Analytics</h1>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if loading}
    <div class="loading">Loading data...</div>
  {:else if sessions.length === 0}
    <div class="empty-state">
      <p>No poker sessions recorded yet.</p>
      <p>Add sessions from the Dashboard to see charts!</p>
    </div>
  {:else}
    <BankrollChart {sessions} />
  {/if}
</div>

<style>
  .charts {
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    padding: 1rem;
    flex: 1;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  h1 {
    margin: 0;
    color: var(--color-text);
    font-size: 1.5rem;
  }

  .error {
    background-color: #fee;
    color: #c33;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .loading {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-secondary);
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--color-text-secondary);
  }

  .empty-state p {
    margin: 0.5rem 0;
  }

  @media (max-width: 768px) {
    .charts {
      padding: 1rem;
    }
  }
</style>
