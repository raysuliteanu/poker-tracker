<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PokerSession } from '../lib/api';

  export let sessions: PokerSession[];

  const dispatch = createEventDispatcher();

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
            <td class="notes">{session.notes || '-'}</td>
            <td class="actions">
              <button
                on:click={() => dispatch('edit', session)}
                class="btn-edit"
                title="Edit session"
                aria-label="Edit session"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
    padding: 1rem;
    text-align: left;
    font-weight: 600;
    color: var(--color-text);
    border-bottom: 2px solid var(--color-border);
  }

  td {
    padding: 1rem;
    color: var(--color-text);
    border-bottom: 1px solid var(--color-border);
  }

  tbody tr:hover {
    background: var(--color-bg-hover);
  }

  .notes {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    padding: 0.5rem;
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

  @media (max-width: 768px) {
    table {
      font-size: 0.875rem;
    }

    th,
    td {
      padding: 0.5rem;
    }

    .notes {
      max-width: 100px;
    }

    .actions {
      flex-direction: column;
    }
  }
</style>
