import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import Dashboard from '../../pages/Dashboard.svelte';
import { mockSessions } from '../mocks';

// Mock the API module
vi.mock('../../lib/api', () => ({
  api: {
    sessions: {
      getAll: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    },
  },
}));

// Mock Chart.js
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

// Mock window.confirm and window.alert
vi.stubGlobal('confirm', vi.fn(() => true));
vi.stubGlobal('alert', vi.fn());

// Import api after mocking
import { api } from '../../lib/api';

describe('Dashboard', () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it('displays loading state initially', async () => {
    vi.mocked(api.sessions.getAll).mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    render(Dashboard);

    expect(screen.getByText('Loading sessions...')).toBeInTheDocument();
  });

  it('renders dashboard header and Add Session button', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Poker Bankroll Tracker')).toBeInTheDocument();
    });
    expect(screen.getByText('Add Session')).toBeInTheDocument();
  });

  it('displays error message when API fails', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ error: 'Failed to load sessions' });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Failed to load sessions')).toBeInTheDocument();
    });
  });

  it('renders stats cards with correct calculations', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      // Total Profit = 50 + (-70) + 120 = 100
      expect(screen.getByText('Total Profit/Loss')).toBeInTheDocument();
      // $100.00 appears multiple times (in stat card and table), so just check at least one exists
      expect(screen.getAllByText('$100.00').length).toBeGreaterThan(0);

      // Total Sessions = 3
      expect(screen.getByText('Total Sessions')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();

      // Total Hours = (120 + 180 + 90) / 60 = 6.5
      expect(screen.getByText('Total Hours')).toBeInTheDocument();
      expect(screen.getByText('6.5')).toBeInTheDocument();
    });
  });

  it('renders hourly rate correctly', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      // Hourly Rate = 100 / 6.5 = ~15.38
      expect(screen.getByText('Hourly Rate')).toBeInTheDocument();
      expect(screen.getByText('$15.38/hr')).toBeInTheDocument();
    });
  });

  it('renders SessionTable with sessions', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      // Check for table headers
      expect(screen.getByText('Date')).toBeInTheDocument();
      expect(screen.getByText('Duration')).toBeInTheDocument();
      expect(screen.getByText('Buy-in')).toBeInTheDocument();
    });
  });

  it('shows empty stats when no sessions', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: [] });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('$0.00')).toBeInTheDocument();
      expect(screen.getByText('0')).toBeInTheDocument();
      expect(screen.getByText('0.0')).toBeInTheDocument();
    });
  });

  it('opens SessionForm when Add Session button clicked', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Poker Bankroll Tracker')).toBeInTheDocument();
    });

    const addButton = screen.getByText('Add Session');
    await fireEvent.click(addButton);

    expect(screen.getByText('Add New Session')).toBeInTheDocument();
  });

  it('closes SessionForm when cancel is clicked', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Poker Bankroll Tracker')).toBeInTheDocument();
    });

    // Open form
    const addButton = screen.getByText('Add Session');
    await fireEvent.click(addButton);
    expect(screen.getByText('Add New Session')).toBeInTheDocument();

    // Cancel form
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);

    // Form should be closed
    await waitFor(() => {
      expect(screen.queryByText('Add New Session')).not.toBeInTheDocument();
    });
  });

  it('calls delete API and reloads when session deleted', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });
    vi.mocked(api.sessions.delete).mockResolvedValue({ data: { message: 'Deleted' } });

    render(Dashboard);

    // Wait for the table to render with delete buttons
    await waitFor(() => {
      expect(screen.getAllByRole('button', { name: /delete session/i }).length).toBeGreaterThan(0);
    });

    // Click delete on first session
    const deleteButtons = screen.getAllByRole('button', { name: /delete session/i });
    await fireEvent.click(deleteButtons[0]);

    expect(window.confirm).toHaveBeenCalled();
    expect(api.sessions.delete).toHaveBeenCalledWith(mockSessions[0].id);
  });

  it('shows alert when delete fails', async () => {
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });
    vi.mocked(api.sessions.delete).mockResolvedValue({ error: 'Delete failed' });

    render(Dashboard);

    // Wait for the table to render with delete buttons
    await waitFor(() => {
      expect(screen.getAllByRole('button', { name: /delete session/i }).length).toBeGreaterThan(0);
    });

    const deleteButtons = screen.getAllByRole('button', { name: /delete session/i });
    await fireEvent.click(deleteButtons[0]);

    await waitFor(() => {
      expect(window.alert).toHaveBeenCalledWith('Delete failed');
    });
  });

  it('does not delete when confirm is cancelled', async () => {
    vi.mocked(window.confirm).mockReturnValueOnce(false);
    vi.mocked(api.sessions.getAll).mockResolvedValue({ data: mockSessions });

    render(Dashboard);

    // Wait for the table to render with delete buttons
    await waitFor(() => {
      expect(screen.getAllByRole('button', { name: /delete session/i }).length).toBeGreaterThan(0);
    });

    const deleteButtons = screen.getAllByRole('button', { name: /delete session/i });
    await fireEvent.click(deleteButtons[0]);

    expect(api.sessions.delete).not.toHaveBeenCalled();
  });
});
