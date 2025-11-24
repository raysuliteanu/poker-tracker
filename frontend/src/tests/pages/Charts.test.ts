import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Charts from '../../pages/Charts.svelte';
import { authStore } from '../../stores/auth';
import { mockUser, mockSessions, successResponse, errorResponse } from '../mocks';
import * as apiModule from '../../lib/api';

// Mock the api module
vi.mock('../../lib/api', async () => {
  const actual = await vi.importActual('../../lib/api');
  return {
    ...actual,
    api: {
      sessions: {
        getAll: vi.fn(),
      },
    },
  };
});

describe('Charts', () => {
  beforeEach(() => {
    authStore.login('test-token', mockUser);
    vi.clearAllMocks();
  });

  it('displays loading state initially', () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse([]));

    render(Charts);

    expect(screen.getByText('Loading data...')).toBeInTheDocument();
  });

  it('loads and displays sessions data', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // Chart should be rendered (BankrollChart component)
    expect(screen.getByText('Charts & Analytics')).toBeInTheDocument();
  });

  it('displays error message when loading fails', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(errorResponse('Failed to load sessions'));

    render(Charts);

    await waitFor(() => {
      expect(screen.getByText('Failed to load sessions')).toBeInTheDocument();
    });
  });

  it('displays stats correctly', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // Check stats cards are displayed
    expect(screen.getByText('Total Profit/Loss')).toBeInTheDocument();
    expect(screen.getByText('Total Sessions')).toBeInTheDocument();
    expect(screen.getByText('Total Hours')).toBeInTheDocument();
    expect(screen.getByText('Hourly Rate')).toBeInTheDocument();
  });

  it('calculates total profit correctly', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // mockSessions: session 1 = +50, session 2 = -70, session 3 = +120
    // Total = 100
    expect(screen.getByText('$100.00')).toBeInTheDocument();
  });

  it('displays empty state when no sessions exist', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse([]));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    expect(screen.getByText('No poker sessions recorded yet.')).toBeInTheDocument();
    expect(screen.getByText('Add sessions from the Dashboard to see charts!')).toBeInTheDocument();
  });

  it('displays total sessions count', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // mockSessions has 3 sessions
    const statValues = screen.getAllByText('3');
    expect(statValues.length).toBeGreaterThan(0);
  });

  it('displays total hours correctly', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // mockSessions: 120 + 180 + 90 = 390 minutes = 6.5 hours
    expect(screen.getByText('6.5')).toBeInTheDocument();
  });

  it('calculates hourly rate correctly', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // Total profit $100 / 6.5 hours = $15.38/hr
    expect(screen.getByText('$15.38/hr')).toBeInTheDocument();
  });

  it('applies profit class for positive total', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    const profitElement = screen.getByText('$100.00');
    expect(profitElement.classList.contains('profit')).toBe(true);
  });

  it('applies loss class for negative total', async () => {
    const losingSessions = [
      {
        ...mockSessions[1], // This session has -70 profit
        id: 'losing-session',
      },
    ];

    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(losingSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    const lossElement = screen.getByText('$-70.00');
    expect(lossElement.classList.contains('loss')).toBe(true);
  });
});
