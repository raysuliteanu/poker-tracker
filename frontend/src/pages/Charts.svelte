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

  $: totalProfit = sessions.reduce((sum, s) => sum + (s.profit || 0), 0);
  $: totalSessions = sessions.length;
  $: totalHours = sessions.reduce((sum, s) => sum + s.duration_minutes / 60, 0);
</script>

<div class="charts">
  <div class="header">
    <h1>Charts & Analytics</h1>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="stats">
    <div class="stat-card">
      <div class="stat-label">Total Profit/Loss</div>
      <div class="stat-value" class:profit={totalProfit >= 0} class:loss={totalProfit < 0}>
        ${totalProfit.toFixed(2)}
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Total Sessions</div>
      <div class="stat-value">{totalSessions}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Total Hours</div>
      <div class="stat-value">{totalHours.toFixed(1)}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Hourly Rate</div>
      <div class="stat-value">
        ${totalHours > 0 ? (totalProfit / totalHours).toFixed(2) : '0.00'}/hr
      </div>
    </div>
  </div>

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

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .stat-card {
    background: var(--color-bg-secondary);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin-bottom: 0.25rem;
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .stat-value.profit {
    color: #10b981;
  }

  .stat-value.loss {
    color: #ef4444;
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

    .stats {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
