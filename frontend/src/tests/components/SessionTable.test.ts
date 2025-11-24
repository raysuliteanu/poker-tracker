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

  it('displays view notes button only for sessions with notes', () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Should have 2 view notes buttons (sessions 1 and 2 have notes)
    const viewNotesButtons = screen.getAllByRole('button', { name: /view notes/i });
    expect(viewNotesButtons).toHaveLength(2);
  });

  it('opens modal with notes when view notes button is clicked', async () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Click the first view notes button
    const viewNotesButtons = screen.getAllByRole('button', { name: /view notes/i });
    await fireEvent.click(viewNotesButtons[0]);

    // Modal should be visible with the notes
    expect(screen.getByText('Good session')).toBeInTheDocument();
    expect(screen.getByText(/Session Notes -/)).toBeInTheDocument();
  });

  it('closes modal when close button is clicked', async () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Open modal
    const viewNotesButtons = screen.getAllByRole('button', { name: /view notes/i });
    await fireEvent.click(viewNotesButtons[0]);

    // Modal should be visible
    expect(screen.getByText('Good session')).toBeInTheDocument();

    // Close modal
    const closeButton = screen.getByRole('button', { name: /close/i });
    await fireEvent.click(closeButton);

    // Modal should be closed - notes text should not be visible
    expect(screen.queryByText('Good session')).not.toBeInTheDocument();
  });

  it('closes modal when clicking overlay', async () => {
    render(SessionTable, { props: { sessions: mockSessions } });

    // Open modal
    const viewNotesButtons = screen.getAllByRole('button', { name: /view notes/i });
    await fireEvent.click(viewNotesButtons[0]);

    // Modal should be visible
    expect(screen.getByText('Good session')).toBeInTheDocument();

    // Click overlay (the modal-overlay div)
    const overlay = screen.getByText('Good session').closest('.modal')?.parentElement;
    if (overlay) {
      await fireEvent.click(overlay);
    }

    // Modal should be closed
    expect(screen.queryByText('Good session')).not.toBeInTheDocument();
  });

  it('does not show view notes button for sessions without notes', () => {
    const sessionsWithoutNotes = [
      {
        ...mockSessions[2],
        notes: null,
      },
      {
        ...mockSessions[2],
        id: 'session-4',
        notes: '',
      },
      {
        ...mockSessions[2],
        id: 'session-5',
        notes: '   ', // whitespace only
      },
    ];

    render(SessionTable, { props: { sessions: sessionsWithoutNotes } });

    // Should have no view notes buttons
    const viewNotesButtons = screen.queryAllByRole('button', { name: /view notes/i });
    expect(viewNotesButtons).toHaveLength(0);
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
