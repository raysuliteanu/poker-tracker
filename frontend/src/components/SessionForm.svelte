<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api, type PokerSession } from '../lib/api';

  export let session: PokerSession | null = null;

  const dispatch = createEventDispatcher();

  let sessionDate = session?.session_date || new Date().toISOString().split('T')[0];
  let durationMinutes = session?.duration_minutes || 0;
  let buyInAmount = session ? parseFloat(session.buy_in_amount) : 0;
  let rebuyAmount = session ? parseFloat(session.rebuy_amount) : 0;
  let cashOutAmount = session ? parseFloat(session.cash_out_amount) : 0;
  let notes = session?.notes || '';

  let error = '';
  let loading = false;

  async function handleSubmit() {
    error = '';
    loading = true;

    if (session) {
      // Update existing session
      const response = await api.sessions.update(session.id, {
        session_date: sessionDate,
        duration_minutes: durationMinutes,
        buy_in_amount: buyInAmount,
        rebuy_amount: rebuyAmount,
        cash_out_amount: cashOutAmount,
        notes: notes || undefined,
      });

      loading = false;

      if (response.error) {
        error = response.error;
      } else {
        dispatch('save');
      }
    } else {
      // Create new session
      const response = await api.sessions.create({
        session_date: sessionDate,
        duration_minutes: durationMinutes,
        buy_in_amount: buyInAmount,
        rebuy_amount: rebuyAmount,
        cash_out_amount: cashOutAmount,
        notes: notes || undefined,
      });

      loading = false;

      if (response.error) {
        error = response.error;
      } else {
        dispatch('save');
      }
    }
  }

  function handleCancel() {
    dispatch('cancel');
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-overlay" on:click={handleCancel}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation>
    <div class="modal-header">
      <h2>{session ? 'Edit Session' : 'Add New Session'}</h2>
      <button class="close-btn" on:click={handleCancel}>&times;</button>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <form on:submit|preventDefault={handleSubmit}>
      <div class="form-group">
        <label for="sessionDate">Session Date</label>
        <input
          id="sessionDate"
          type="date"
          bind:value={sessionDate}
          required
        />
      </div>

      <div class="form-group">
        <label for="durationMinutes">Duration (minutes)</label>
        <input
          id="durationMinutes"
          type="number"
          bind:value={durationMinutes}
          min="1"
          required
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="buyInAmount">Buy-in ($)</label>
          <input
            id="buyInAmount"
            type="number"
            step="0.01"
            bind:value={buyInAmount}
            min="0"
            required
          />
        </div>

        <div class="form-group">
          <label for="rebuyAmount">Rebuy ($)</label>
          <input
            id="rebuyAmount"
            type="number"
            step="0.01"
            bind:value={rebuyAmount}
            min="0"
          />
        </div>
      </div>

      <div class="form-group">
        <label for="cashOutAmount">Cash Out ($)</label>
        <input
          id="cashOutAmount"
          type="number"
          step="0.01"
          bind:value={cashOutAmount}
          min="0"
          required
        />
      </div>

      <div class="form-group">
        <label for="notes">Notes (optional)</label>
        <textarea
          id="notes"
          bind:value={notes}
          rows="3"
          placeholder="Add any notes about this session..."
        ></textarea>
      </div>

      <div class="form-actions">
        <button type="button" on:click={handleCancel} class="btn-secondary">
          Cancel
        </button>
        <button type="submit" disabled={loading} class="btn-primary">
          {loading ? 'Saving...' : 'Save Session'}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
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
    padding: 1rem;
  }

  .modal {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    width: 100%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid var(--color-border);
  }

  h2 {
    margin: 0;
    color: var(--color-text);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 2rem;
    color: var(--color-text-secondary);
    cursor: pointer;
    line-height: 1;
    padding: 0;
    width: 2rem;
    height: 2rem;
  }

  .close-btn:hover {
    color: var(--color-text);
  }

  form {
    padding: 1.5rem;
  }

  .error {
    background-color: #fee;
    color: #c33;
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--color-text);
    font-weight: 500;
  }

  input,
  textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 1rem;
    background: var(--color-bg);
    color: var(--color-text);
    box-sizing: border-box;
    font-family: inherit;
  }

  input:focus,
  textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  textarea {
    resize: vertical;
  }

  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background-color: var(--color-primary);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: var(--color-primary-dark);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-secondary {
    background-color: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .btn-secondary:hover {
    background-color: var(--color-bg-hover);
  }

  @media (max-width: 768px) {
    .modal {
      max-width: 100%;
      max-height: 100vh;
      border-radius: 0;
    }

    .form-row {
      grid-template-columns: 1fr;
    }
  }
</style>
