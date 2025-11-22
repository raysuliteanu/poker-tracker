import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SessionTable from '../../components/SessionTable.svelte';
import { mockSessions } from '../mocks';

// Mock createEventDispatcher
const mockDispatch = vi.fn();
vi.mock('svelte', async () => {
  const actual = await vi.importActual('svelte');
  return {
    ...actual,
    createEventDispatcher: () => mockDispatch,
  };
});

beforeEach(() => {
  mockDispatch.mockClear();
});

describe('SessionTable', () => {
  it('displays empty state when no sessions', () => {
    render(SessionTable, { props: { sessions: [] } });

    expect(screen.getByText('No poker sessions recorded yet.')).toBeInTheDocument();
    expect(screen.getByText(/Click "Add Session"/)).toBeInTheDocument();
  });

  it('renders table with sessions', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Check table headers
    expect(screen.getByText('Date')).toBeInTheDocument();
    expect(screen.getByText('Duration')).toBeInTheDocument();
    expect(screen.getByText('Buy-in')).toBeInTheDocument();
    expect(screen.getByText('Rebuy')).toBeInTheDocument();
    expect(screen.getByText('Cash Out')).toBeInTheDocument();
    expect(screen.getByText('Profit/Loss')).toBeInTheDocument();
    expect(screen.getByText('Notes')).toBeInTheDocument();
    expect(screen.getByText('Actions')).toBeInTheDocument();
  });

  it('formats duration correctly', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // 120 minutes = 2h 0m
    expect(screen.getByText('2h 0m')).toBeInTheDocument();
    // 180 minutes = 3h 0m
    expect(screen.getByText('3h 0m')).toBeInTheDocument();
    // 90 minutes = 1h 30m
    expect(screen.getByText('1h 30m')).toBeInTheDocument();
  });

  it('formats money values correctly', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Buy-in amounts
    expect(screen.getAllByText('$100.00').length).toBeGreaterThan(0);
    expect(screen.getByText('$200.00')).toBeInTheDocument();
  });

  it('calculates and displays profit correctly', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Session 1: 150 - (100 + 0) = $50 profit (also appears as rebuy for session 2)
    const profitElements = screen.getAllByText('$50.00');
    expect(profitElements.length).toBeGreaterThanOrEqual(1);
    // Session 3: 220 - (100 + 0) = $120 profit
    expect(screen.getByText('$120.00')).toBeInTheDocument();
  });

  it('calculates and displays loss correctly', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Session 2: 180 - (200 + 50) = -$70 loss
    expect(screen.getByText('$-70.00')).toBeInTheDocument();
  });

  it('displays notes or dash for empty notes', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    expect(screen.getByText('Good session')).toBeInTheDocument();
    expect(screen.getByText('Tough table')).toBeInTheDocument();
    expect(screen.getByText('-')).toBeInTheDocument(); // Session 3 has null notes
  });

  it('renders edit and delete buttons for each session', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    const editButtons = screen.getAllByRole('button', { name: /edit session/i });
    const deleteButtons = screen.getAllByRole('button', { name: /delete session/i });

    expect(editButtons).toHaveLength(3);
    expect(deleteButtons).toHaveLength(3);
  });

  it('dispatches edit event when edit button clicked', async () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    const editButtons = screen.getAllByRole('button', { name: /edit session/i });
    await fireEvent.click(editButtons[0]);

    expect(mockDispatch).toHaveBeenCalledWith('edit', mockSessions[0]);
  });

  it('dispatches delete event when delete button clicked', async () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    const deleteButtons = screen.getAllByRole('button', { name: /delete session/i });
    await fireEvent.click(deleteButtons[1]);

    expect(mockDispatch).toHaveBeenCalledWith('delete', mockSessions[1].id);
  });
});
