<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import type { PokerSession } from '../lib/api';

  export let sessions: PokerSession[];

  Chart.register(...registerables);

  let canvas: HTMLCanvasElement;
  let chart: Chart | null = null;
  let timeRange: 'week' | 'month' | 'quarter' | 'year' | 'all' = 'all';

  $: filteredSessions = filterSessionsByTimeRange(sessions, timeRange);

  function filterSessionsByTimeRange(
    sessions: PokerSession[],
    range: string
  ): PokerSession[] {
    if (range === 'all') return sessions;

    const now = new Date();
    const cutoffDate = new Date();

    switch (range) {
      case 'week':
        cutoffDate.setDate(now.getDate() - 7);
        break;
      case 'month':
        cutoffDate.setMonth(now.getMonth() - 1);
        break;
      case 'quarter':
        cutoffDate.setMonth(now.getMonth() - 3);
        break;
      case 'year':
        cutoffDate.setFullYear(now.getFullYear() - 1);
        break;
    }

    return sessions.filter(
      (s) => new Date(s.session_date) >= cutoffDate
    );
  }

  function calculateChartData() {
    if (filteredSessions.length === 0) {
      return { labels: [], data: [] };
    }

    // Sort sessions by date
    const sorted = [...filteredSessions].sort(
      (a, b) =>
        new Date(a.session_date).getTime() - new Date(b.session_date).getTime()
    );

    const labels: string[] = [];
    const data: number[] = [];
    let runningTotal = 0;

    sorted.forEach((session) => {
      const buyIn = parseFloat(session.buy_in_amount);
      const rebuy = parseFloat(session.rebuy_amount);
      const cashOut = parseFloat(session.cash_out_amount);
      const profit = cashOut - (buyIn + rebuy);

      runningTotal += profit;

      labels.push(new Date(session.session_date).toLocaleDateString());
      data.push(runningTotal);
    });

    return { labels, data };
  }

  function renderChart() {
    if (!canvas) return;

    const { labels, data } = calculateChartData();

    if (chart) {
      chart.destroy();
    }

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    chart = new Chart(ctx, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Bankroll',
            data,
            borderColor: 'rgb(59, 130, 246)',
            backgroundColor: 'rgba(59, 130, 246, 0.1)',
            tension: 0.3,
            fill: true,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: false,
          },
          tooltip: {
            callbacks: {
              label: (context) => `Profit/Loss: $${context.parsed.y.toFixed(2)}`,
            },
          },
        },
        scales: {
          y: {
            ticks: {
              callback: (value) => `$${value}`,
            },
          },
        },
      },
    });
  }

  onMount(() => {
    renderChart();
  });

  afterUpdate(() => {
    renderChart();
  });
</script>

<div class="chart-container">
  <div class="chart-header">
    <h3>Bankroll Over Time</h3>
    <div class="time-range-selector">
      <button
        class:active={timeRange === 'week'}
        on:click={() => (timeRange = 'week')}
      >
        Week
      </button>
      <button
        class:active={timeRange === 'month'}
        on:click={() => (timeRange = 'month')}
      >
        Month
      </button>
      <button
        class:active={timeRange === 'quarter'}
        on:click={() => (timeRange = 'quarter')}
      >
        Quarter
      </button>
      <button
        class:active={timeRange === 'year'}
        on:click={() => (timeRange = 'year')}
      >
        Year
      </button>
      <button
        class:active={timeRange === 'all'}
        on:click={() => (timeRange = 'all')}
      >
        All Time
      </button>
    </div>
  </div>

  <div class="chart-wrapper">
    {#if filteredSessions.length === 0}
      <div class="empty-chart">
        No sessions found for the selected time range.
      </div>
    {:else}
      <canvas bind:this={canvas} />
    {/if}
  </div>
</div>

<style>
  .chart-container {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .chart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    gap: 1rem;
  }

  h3 {
    margin: 0;
    color: var(--color-text);
  }

  .time-range-selector {
    display: flex;
    gap: 0.5rem;
  }

  .time-range-selector button {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border);
    background-color: var(--color-bg);
    color: var(--color-text);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.875rem;
  }

  .time-range-selector button:hover {
    background-color: var(--color-bg-hover);
  }

  .time-range-selector button.active {
    background-color: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }

  .chart-wrapper {
    position: relative;
    height: 300px;
  }

  .empty-chart {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .chart-header {
      flex-direction: column;
      align-items: stretch;
    }

    .time-range-selector {
      flex-wrap: wrap;
      justify-content: center;
    }

    .time-range-selector button {
      flex: 1;
      min-width: 70px;
    }
  }
</style>
