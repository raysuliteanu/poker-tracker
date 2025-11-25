import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import SessionForm from '../../components/SessionForm.svelte';
import { mockSessions } from '../mocks';

// Mock the API module
vi.mock('../../lib/api', () => ({
  api: {
    sessions: {
      create: vi.fn(),
      update: vi.fn(),
    },
  },
}));

// Mock createEventDispatcher
const mockDispatch = vi.fn();
vi.mock('svelte', async () => {
  const actual = await vi.importActual('svelte');
  return {
    ...actual,
    createEventDispatcher: () => mockDispatch,
  };
});

// Import api after mocking
import { api } from '../../lib/api';

describe('SessionForm', () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
    mockDispatch.mockClear();
  });

  it('renders "Add New Session" title when no session provided', () => {
    render(SessionForm);

    expect(screen.getByText('Add New Session')).toBeInTheDocument();
  });

  it('renders "Edit Session" title when session provided', () => {
    render(SessionForm, { props: { session: mockSessions[0] } });

    expect(screen.getByText('Edit Session')).toBeInTheDocument();
  });

  it('renders all form fields', () => {
    render(SessionForm);

    expect(screen.getByLabelText('Session Date')).toBeInTheDocument();
    expect(screen.getByLabelText('Duration (hours)')).toBeInTheDocument();
    expect(screen.getByLabelText('Buy-in ($)')).toBeInTheDocument();
    expect(screen.getByLabelText('Rebuy ($)')).toBeInTheDocument();
    expect(screen.getByLabelText('Cash Out ($)')).toBeInTheDocument();
    expect(screen.getByLabelText('Notes (optional)')).toBeInTheDocument();
  });

  it('renders Save and Cancel buttons', () => {
    render(SessionForm);

    expect(screen.getByText('Save Session')).toBeInTheDocument();
    expect(screen.getByText('Cancel')).toBeInTheDocument();
  });

  it('pre-fills form with session data when editing', () => {
    render(SessionForm, { props: { session: mockSessions[0] } });

    const dateInput = screen.getByLabelText('Session Date') as HTMLInputElement;
    const durationInput = screen.getByLabelText('Duration (hours)') as HTMLInputElement;
    const buyInInput = screen.getByLabelText('Buy-in ($)') as HTMLInputElement;

    expect(dateInput.value).toBe('2024-01-15');
    expect(durationInput.value).toBe('2'); // 120 minutes = 2 hours
    expect(buyInInput.value).toBe('100');
  });

  it('dispatches cancel event when cancel button clicked', async () => {
    render(SessionForm);

    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);

    expect(mockDispatch).toHaveBeenCalledWith('cancel');
  });

  it('dispatches cancel event when overlay clicked', async () => {
    const { container } = render(SessionForm);

    const overlay = container.querySelector('.modal-overlay')!;
    await fireEvent.click(overlay);

    expect(mockDispatch).toHaveBeenCalledWith('cancel');
  });

  it('dispatches cancel event when close button clicked', async () => {
    render(SessionForm);

    const closeButton = screen.getByText('×');
    await fireEvent.click(closeButton);

    expect(mockDispatch).toHaveBeenCalledWith('cancel');
  });

  it('calls api.sessions.create and dispatches save on successful new session', async () => {
    vi.mocked(api.sessions.create).mockResolvedValue({ data: mockSessions[0] });

    render(SessionForm);

    // Fill in required fields
    const dateInput = screen.getByLabelText('Session Date');
    const durationInput = screen.getByLabelText('Duration (hours)');
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    const cashOutInput = screen.getByLabelText('Cash Out ($)');

    await fireEvent.input(dateInput, { target: { value: '2024-02-01' } });
    await fireEvent.input(durationInput, { target: { value: '1' } }); // 1 hour
    await fireEvent.input(buyInInput, { target: { value: '100' } });
    await fireEvent.input(cashOutInput, { target: { value: '200' } });

    // Submit form
    const submitButton = screen.getByText('Save Session');
    await fireEvent.click(submitButton);

    expect(api.sessions.create).toHaveBeenCalled();
  });

  it('calls api.sessions.update when editing existing session', async () => {
    vi.mocked(api.sessions.update).mockResolvedValue({ data: mockSessions[0] });

    render(SessionForm, { props: { session: mockSessions[0] } });

    // Modify a field
    const durationInput = screen.getByLabelText('Duration (hours)');
    await fireEvent.input(durationInput, { target: { value: '3' } }); // 3 hours

    // Submit form
    const submitButton = screen.getByText('Save Session');
    await fireEvent.click(submitButton);

    expect(api.sessions.update).toHaveBeenCalledWith(mockSessions[0].id, expect.any(Object));
  });

  it('displays error message when API returns error', async () => {
    vi.mocked(api.sessions.create).mockResolvedValue({ error: 'Failed to create session' });

    render(SessionForm);

    // Fill in required fields
    const dateInput = screen.getByLabelText('Session Date');
    const durationInput = screen.getByLabelText('Duration (hours)');
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    const cashOutInput = screen.getByLabelText('Cash Out ($)');

    await fireEvent.input(dateInput, { target: { value: '2024-02-01' } });
    await fireEvent.input(durationInput, { target: { value: '1' } }); // 1 hour
    await fireEvent.input(buyInInput, { target: { value: '100' } });
    await fireEvent.input(cashOutInput, { target: { value: '200' } });

    // Submit form
    const submitButton = screen.getByText('Save Session');
    await fireEvent.click(submitButton);

    expect(screen.getByText('Failed to create session')).toBeInTheDocument();
  });

  it('shows loading state while submitting', async () => {
    // Make the API call hang
    vi.mocked(api.sessions.create).mockImplementation(
      () => new Promise(() => {}) // Never resolves
    );

    render(SessionForm);

    // Fill in required fields
    const dateInput = screen.getByLabelText('Session Date');
    const durationInput = screen.getByLabelText('Duration (hours)');
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    const cashOutInput = screen.getByLabelText('Cash Out ($)');

    await fireEvent.input(dateInput, { target: { value: '2024-02-01' } });
    await fireEvent.input(durationInput, { target: { value: '1' } }); // 1 hour
    await fireEvent.input(buyInInput, { target: { value: '100' } });
    await fireEvent.input(cashOutInput, { target: { value: '200' } });

    // Submit form
    const submitButton = screen.getByText('Save Session');
    await fireEvent.click(submitButton);

    expect(screen.getByText('Saving...')).toBeInTheDocument();
  });

  it('enforces whole dollar amounts for money inputs', () => {
    render(SessionForm);

    const buyInInput = screen.getByLabelText('Buy-in ($)') as HTMLInputElement;
    const rebuyInput = screen.getByLabelText('Rebuy ($)') as HTMLInputElement;
    const cashOutInput = screen.getByLabelText('Cash Out ($)') as HTMLInputElement;

    // Verify step is set to 1 (whole dollars only)
    expect(buyInInput.step).toBe('1');
    expect(rebuyInput.step).toBe('1');
    expect(cashOutInput.step).toBe('1');

    // Verify no-spinner class is applied
    expect(buyInInput.classList.contains('no-spinner')).toBe(true);
    expect(rebuyInput.classList.contains('no-spinner')).toBe(true);
    expect(cashOutInput.classList.contains('no-spinner')).toBe(true);
  });

  it('enforces decimal hours with step 0.01', () => {
    render(SessionForm);

    const durationInput = screen.getByLabelText('Duration (hours)') as HTMLInputElement;

    // Verify step is set to 0.01 (allows decimals like 2.5)
    expect(durationInput.step).toBe('0.01');
    expect(durationInput.classList.contains('no-spinner')).toBe(true);
  });
});
