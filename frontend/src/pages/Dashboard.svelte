<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api, type PokerSession } from '../lib/api';
  import SessionTable from '../components/SessionTable.svelte';
  import SessionForm from '../components/SessionForm.svelte';
  import BankrollChart from '../components/BankrollChart.svelte';

  let sessions: PokerSession[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let editingSession: PokerSession | null = null;

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

  function handleAddNew() {
    editingSession = null;
    showForm = true;
  }

  function handleEdit(session: PokerSession) {
    editingSession = session;
    showForm = true;
  }

  function handleCloseForm() {
    showForm = false;
    editingSession = null;
  }

  async function handleSave() {
    await loadSessions();
    handleCloseForm();
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this session?')) {
      return;
    }

    const response = await api.sessions.delete(id);

    if (response.error) {
      alert(response.error);
    } else {
      await loadSessions();
    }
  }

  $: totalProfit = sessions.reduce((sum, s) => sum + (s.profit || 0), 0);
  $: totalSessions = sessions.length;
  $: totalHours = sessions.reduce((sum, s) => sum + s.duration_minutes / 60, 0);
</script>

<div class="dashboard">
  <div class="header">
    <h1>Poker Bankroll Tracker</h1>
    <button on:click={handleAddNew} class="btn-primary">
      Add Session
    </button>
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

  <BankrollChart {sessions} />

  {#if loading}
    <div class="loading">Loading sessions...</div>
  {:else}
    <SessionTable {sessions} on:edit={(e) => handleEdit(e.detail)} on:delete={(e) => handleDelete(e.detail)} />
  {/if}

  {#if showForm}
    <SessionForm session={editingSession} on:save={handleSave} on:cancel={handleCloseForm} />
  {/if}
</div>

<style>
  .dashboard {
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
    flex: 1;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  h1 {
    margin: 0;
    color: var(--color-text);
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

  .btn-primary:hover {
    background-color: var(--color-primary-dark);
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
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: var(--color-bg-secondary);
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .stat-label {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin-bottom: 0.5rem;
  }

  .stat-value {
    font-size: 1.75rem;
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

  @media (max-width: 768px) {
    .dashboard {
      padding: 1rem;
    }

    .header {
      flex-direction: column;
      gap: 1rem;
      align-items: stretch;
    }

    .stats {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
