import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import BankrollChart from '../../components/BankrollChart.svelte';
import { mockSessions } from '../mocks';
import type { PokerSession } from '../../lib/api';

// Mock Chart.js to avoid canvas rendering issues
vi.mock('chart.js', () => {
  class MockChart {
    static register = vi.fn();
    destroy = vi.fn();
    update = vi.fn();
    constructor() {}
  }
  return {
    Chart: MockChart,
    registerables: [],
  };
});

describe('BankrollChart', () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it('renders chart container', () => {
    render(BankrollChart, { props: { sessions: mockSessions } });

    expect(screen.getByText('Bankroll Over Time')).toBeInTheDocument();
  });

  it('renders time range selector buttons', () => {
    render(BankrollChart, { props: { sessions: mockSessions } });

    expect(screen.getByText('Week')).toBeInTheDocument();
    expect(screen.getByText('Month')).toBeInTheDocument();
    expect(screen.getByText('Quarter')).toBeInTheDocument();
    expect(screen.getByText('Year')).toBeInTheDocument();
    expect(screen.getByText('All Time')).toBeInTheDocument();
  });

  it('defaults to "All Time" selected', () => {
    render(BankrollChart, { props: { sessions: mockSessions } });

    const allTimeButton = screen.getByText('All Time');
    expect(allTimeButton.classList.contains('active')).toBe(true);
  });

  it('switches active state when time range button clicked', async () => {
    render(BankrollChart, { props: { sessions: mockSessions } });

    const weekButton = screen.getByText('Week');
    const allTimeButton = screen.getByText('All Time');

    // Initially All Time is active
    expect(allTimeButton.classList.contains('active')).toBe(true);
    expect(weekButton.classList.contains('active')).toBe(false);

    // Click Week
    await fireEvent.click(weekButton);

    // Now Week should be active
    expect(weekButton.classList.contains('active')).toBe(true);
    expect(allTimeButton.classList.contains('active')).toBe(false);
  });

  it('displays empty state when no sessions', () => {
    render(BankrollChart, { props: { sessions: [] } });

    expect(screen.getByText('No sessions found for the selected time range.')).toBeInTheDocument();
  });

  it('displays empty state when no sessions match time filter', async () => {
    // Sessions are from Jan 2024, selecting week filter shouldn't match any
    render(BankrollChart, { props: { sessions: mockSessions } });

    const weekButton = screen.getByText('Week');
    await fireEvent.click(weekButton);

    // Since mock sessions are from Jan 2024, filtering by "week" (last 7 days) should show no data
    expect(screen.getByText('No sessions found for the selected time range.')).toBeInTheDocument();
  });

  it('renders canvas when sessions exist for selected time range', () => {
    const { container } = render(BankrollChart, { props: { sessions: mockSessions } });

    // With "All Time" selected and sessions available, canvas should render
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('filters sessions correctly by month', async () => {
    // Create sessions with recent dates for testing
    const recentDate = new Date();
    recentDate.setDate(recentDate.getDate() - 15); // 15 days ago

    const recentSessions: PokerSession[] = [
      {
        ...mockSessions[0],
        session_date: recentDate.toISOString().split('T')[0],
      },
    ];

    const { container } = render(BankrollChart, { props: { sessions: recentSessions } });

    // Select Month filter
    const monthButton = screen.getByText('Month');
    await fireEvent.click(monthButton);

    // Should still show canvas since session is within last month
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('updates chart when time range changes', async () => {
    render(BankrollChart, { props: { sessions: mockSessions } });

    // Click through different time ranges
    const quarterButton = screen.getByText('Quarter');
    await fireEvent.click(quarterButton);
    expect(quarterButton.classList.contains('active')).toBe(true);

    const yearButton = screen.getByText('Year');
    await fireEvent.click(yearButton);
    expect(yearButton.classList.contains('active')).toBe(true);
  });
});
