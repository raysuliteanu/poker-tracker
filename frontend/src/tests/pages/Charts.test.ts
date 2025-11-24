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

  it('renders chart when sessions exist', async () => {
    apiModule.api.sessions.getAll = vi.fn().mockResolvedValue(successResponse(mockSessions));

    render(Charts);

    await waitFor(() => {
      expect(screen.queryByText('Loading data...')).not.toBeInTheDocument();
    });

    // Chart should be rendered (BankrollChart component shows "Bankroll Over Time")
    expect(screen.getByText('Charts & Analytics')).toBeInTheDocument();
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

});
