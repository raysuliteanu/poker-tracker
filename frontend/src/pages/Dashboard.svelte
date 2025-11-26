<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api, type PokerSession } from '../lib/api';
  import SessionTable from '../components/SessionTable.svelte';
  import SessionForm from '../components/SessionForm.svelte';

  let sessions: PokerSession[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let editingSession: PokerSession | null = null;
  let exportTimeRange = 'all';
  let exporting = false;
  let showExportMenu = false;

  onMount(() => {
    const init = async () => {
      await loadSessions();
    };
    init();

    // Add global click handler for closing export menu
    document.addEventListener('click', handleClickOutside);

    // Cleanup on unmount
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
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

  function toggleExportMenu() {
    showExportMenu = !showExportMenu;
  }

  async function handleExport(timeRange: string) {
    showExportMenu = false;
    exporting = true;
    const response = await api.sessions.export(timeRange);
    exporting = false;

    if (response.error) {
      alert(response.error);
      return;
    }

    if (response.data) {
      // Create download link
      const url = URL.createObjectURL(response.data);
      const link = document.createElement('a');
      link.href = url;
      link.download = `poker-sessions-${timeRange}.csv`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  }

  // Close export menu when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (showExportMenu && !target.closest('.toolbar-export')) {
      showExportMenu = false;
    }
  }

  $: totalProfit = sessions.reduce((sum, s) => sum + (s.profit || 0), 0);
  $: totalSessions = sessions.length;
  $: totalHours = sessions.reduce((sum, s) => sum + s.duration_minutes / 60, 0);
</script>

<div class="dashboard">
  <div class="header">
    <h1>Poker Bankroll Tracker</h1>
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
        ${totalHours > 0 ? (totalProfit / totalHours).toFixed(2) : '0.00'}
      </div>
    </div>
  </div>

  <div class="table-toolbar">
    <button
      on:click={handleAddNew}
      class="toolbar-btn"
      title="Add Session"
      aria-label="Add Session"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
    </button>

    <div class="toolbar-export">
      <button
        on:click={toggleExportMenu}
        class="toolbar-btn"
        class:active={showExportMenu}
        title="Export Sessions"
        aria-label="Export Sessions"
        disabled={exporting || loading}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
      </button>

      {#if showExportMenu}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="export-dropdown" on:click|stopPropagation on:keydown|stopPropagation>
          <button on:click={() => handleExport('7days')} class="export-option">
            Last 7 Days
          </button>
          <button on:click={() => handleExport('30days')} class="export-option">
            Last 30 Days
          </button>
          <button on:click={() => handleExport('90days')} class="export-option">
            Last 90 Days
          </button>
          <button on:click={() => handleExport('1year')} class="export-option">
            Last Year
          </button>
          <button on:click={() => handleExport('all')} class="export-option">
            All Sessions
          </button>
        </div>
      {/if}
    </div>
  </div>

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
    padding: 1rem;
    flex: 1;
  }

  .header {
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

  .table-toolbar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    padding: 0.5rem;
    background: var(--color-bg-secondary);
    border-radius: 6px;
    border: 1px solid var(--color-border);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text);
    cursor: pointer;
    transition: all 0.2s;
  }

  .toolbar-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .toolbar-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toolbar-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .toolbar-btn svg {
    display: block;
  }

  .toolbar-export {
    position: relative;
  }

  .export-dropdown {
    position: absolute;
    top: calc(100% + 0.25rem);
    left: 0;
    min-width: max-content;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    z-index: 100;
    overflow: hidden;
  }

  .export-option {
    display: block;
    width: 100%;
    padding: 0.625rem 0.875rem;
    background: none;
    border: none;
    text-align: left;
    font-size: 0.875rem;
    color: var(--color-text);
    cursor: pointer;
    transition: background-color 0.15s;
    white-space: nowrap;
  }

  .export-option:hover {
    background: var(--color-bg-hover);
  }

  .export-option:not(:last-child) {
    border-bottom: 1px solid var(--color-border);
  }

  @media (max-width: 768px) {
    .dashboard {
      padding: 1rem;
    }

    .stats {
      grid-template-columns: 1fr 1fr;
    }

    .table-toolbar {
      justify-content: flex-start;
    }
  }
</style>
