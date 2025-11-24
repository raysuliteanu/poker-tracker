<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { PokerSession } from '../lib/api';

  export let sessions: PokerSession[];

  const dispatch = createEventDispatcher();

  let showNotesModal = false;
  let currentNotes = '';
  let currentSessionDate = '';

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString();
  }

  function formatDuration(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours}h ${mins}m`;
  }

  function formatMoney(amount: string): string {
    return `$${parseFloat(amount).toFixed(2)}`;
  }

  function calculateProfit(session: PokerSession): number {
    const buyIn = parseFloat(session.buy_in_amount);
    const rebuy = parseFloat(session.rebuy_amount);
    const cashOut = parseFloat(session.cash_out_amount);
    return cashOut - (buyIn + rebuy);
  }

  function viewNotes(session: PokerSession) {
    currentNotes = session.notes || '';
    currentSessionDate = formatDate(session.session_date);
    showNotesModal = true;
  }

  function closeNotesModal() {
    showNotesModal = false;
    currentNotes = '';
    currentSessionDate = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && showNotesModal) {
      closeNotesModal();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="table-container">
  {#if sessions.length === 0}
    <div class="empty-state">
      <p>No poker sessions recorded yet.</p>
      <p>Click "Add Session" to start tracking your bankroll!</p>
    </div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>Duration</th>
          <th>Buy-in</th>
          <th>Rebuy</th>
          <th>Cash Out</th>
          <th>Profit/Loss</th>
          <th>Notes</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sessions as session (session.id)}
          {@const profit = calculateProfit(session)}
          <tr>
            <td>{formatDate(session.session_date)}</td>
            <td>{formatDuration(session.duration_minutes)}</td>
            <td>{formatMoney(session.buy_in_amount)}</td>
            <td>{formatMoney(session.rebuy_amount)}</td>
            <td>{formatMoney(session.cash_out_amount)}</td>
            <td class:profit={profit >= 0} class:loss={profit < 0}>
              {formatMoney(profit.toString())}
            </td>
            <td class="notes">
              {#if session.notes && session.notes.trim()}
                <button
                  on:click={() => viewNotes(session)}
                  class="btn-view-notes"
                  title="View notes"
                  aria-label="View notes"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                    <circle cx="12" cy="12" r="3"/>
                  </svg>
                </button>
              {/if}
            </td>
            <td class="actions">
              <button
                on:click={() => dispatch('edit', session)}
                class="btn-edit"
                title="Edit session"
                aria-label="Edit session"
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>
                  <path d="m15 5 4 4"/>
                </svg>
              </button>
              <button
                on:click={() => dispatch('delete', session.id)}
                class="btn-delete"
                title="Delete session"
                aria-label="Delete session"
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 6h18"/>
                  <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
                  <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
                  <line x1="10" x2="10" y1="11" y2="17"/>
                  <line x1="14" x2="14" y1="11" y2="17"/>
                </svg>
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

{#if showNotesModal}
  <div class="modal-overlay" on:click={closeNotesModal} on:keydown={(e) => e.key === 'Enter' && closeNotesModal()} role="button" tabindex="0">
    <div class="modal" on:click|stopPropagation on:keydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>Session Notes - {currentSessionDate}</h3>
        <button on:click={closeNotesModal} class="btn-close" aria-label="Close">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      <div class="modal-body">
        <p>{currentNotes}</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .table-container {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    overflow-x: auto;
  }

  .empty-state {
    padding: 3rem;
    text-align: center;
    color: var(--color-text-secondary);
  }

  .empty-state p {
    margin: 0.5rem 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: var(--color-bg-hover);
  }

  th {
    padding: 0.5rem 0.75rem;
    text-align: left;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text);
    border-bottom: 2px solid var(--color-border);
  }

  td {
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    color: var(--color-text);
    border-bottom: 1px solid var(--color-border);
  }

  tbody tr:hover {
    background: var(--color-bg-hover);
  }

  .notes {
    text-align: center;
    width: 60px;
  }

  .btn-view-notes {
    background-color: transparent;
    color: var(--color-text-secondary);
    padding: 0.25rem;
    margin: 0 auto;
  }

  .btn-view-notes:hover {
    color: var(--color-primary);
    background-color: transparent;
  }

  .btn-view-notes svg {
    width: 16px;
    height: 16px;
  }

  .profit {
    color: #10b981;
    font-weight: 600;
  }

  .loss {
    color: #ef4444;
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.35rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  button svg {
    display: block;
    width: 14px;
    height: 14px;
  }

  .btn-edit {
    background-color: var(--color-primary);
    color: white;
  }

  .btn-edit:hover {
    background-color: var(--color-primary-dark);
  }

  .btn-delete {
    background-color: #ef4444;
    color: white;
  }

  .btn-delete:hover {
    background-color: #dc2626;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow: auto;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1rem;
    color: var(--color-text);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close:hover {
    color: var(--color-text);
  }

  .btn-close svg {
    width: 20px;
    height: 20px;
  }

  .modal-body {
    padding: 1rem;
  }

  .modal-body p {
    margin: 0;
    color: var(--color-text);
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
    text-align: left;
  }

  @media (max-width: 768px) {
    table {
      font-size: 0.875rem;
    }

    th,
    td {
      padding: 0.5rem;
    }

    .actions {
      flex-direction: column;
    }
  }
</style>
