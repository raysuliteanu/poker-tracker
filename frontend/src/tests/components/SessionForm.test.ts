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

  it('closes immediately on ESC key when form is clean', async () => {
    render(SessionForm);

    // Press ESC key
    await fireEvent.keyDown(window, { key: 'Escape' });

    expect(mockDispatch).toHaveBeenCalledWith('cancel');
  });

  it('shows confirmation dialog on ESC key when form is dirty', async () => {
    render(SessionForm);

    // Make the form dirty by changing a value
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    await fireEvent.input(buyInInput, { target: { value: '100' } });

    // Press ESC key
    await fireEvent.keyDown(window, { key: 'Escape' });

    // Should not dispatch cancel yet
    expect(mockDispatch).not.toHaveBeenCalled();

    // Confirmation dialog should be visible
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();
    expect(
      screen.getByText('You have unsaved changes. Are you sure you want to close this dialog?')
    ).toBeInTheDocument();
  });

  it('shows confirmation dialog when clicking cancel button with dirty form', async () => {
    render(SessionForm);

    // Make the form dirty
    const durationInput = screen.getByLabelText('Duration (hours)');
    await fireEvent.input(durationInput, { target: { value: '2' } });

    // Click cancel button
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);

    // Should not dispatch cancel yet
    expect(mockDispatch).not.toHaveBeenCalled();

    // Confirmation dialog should be visible
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();
  });

  it('shows confirmation dialog when clicking close button with dirty form', async () => {
    render(SessionForm);

    // Make the form dirty
    const notesInput = screen.getByLabelText('Notes (optional)');
    await fireEvent.input(notesInput, { target: { value: 'Some notes' } });

    // Click close button (×)
    const closeButton = screen.getByText('×');
    await fireEvent.click(closeButton);

    // Should not dispatch cancel yet
    expect(mockDispatch).not.toHaveBeenCalled();

    // Confirmation dialog should be visible
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();
  });

  it('shows confirmation dialog when clicking overlay with dirty form', async () => {
    const { container } = render(SessionForm);

    // Make the form dirty
    const cashOutInput = screen.getByLabelText('Cash Out ($)');
    await fireEvent.input(cashOutInput, { target: { value: '500' } });

    // Click overlay
    const overlay = container.querySelector('.modal-overlay')!;
    await fireEvent.click(overlay);

    // Should not dispatch cancel yet
    expect(mockDispatch).not.toHaveBeenCalled();

    // Confirmation dialog should be visible
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();
  });

  it('stays on form when clicking Cancel in confirmation dialog', async () => {
    render(SessionForm);

    // Make the form dirty
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    await fireEvent.input(buyInInput, { target: { value: '100' } });

    // Press ESC key to show confirmation
    await fireEvent.keyDown(window, { key: 'Escape' });

    // Click "Cancel" button in confirmation dialog (the primary button with data-default)
    const cancelButtons = screen.getAllByText('Cancel');
    const confirmDialogCancelButton = cancelButtons.find(btn => btn.hasAttribute('data-default'));
    await fireEvent.click(confirmDialogCancelButton!);

    // Should not dispatch cancel
    expect(mockDispatch).not.toHaveBeenCalled();

    // Confirmation dialog should be hidden
    expect(screen.queryByText('Unsaved Changes')).not.toBeInTheDocument();

    // Form should still be visible
    expect(screen.getByText('Add New Session')).toBeInTheDocument();
  });

  it('closes form when clicking Continue in confirmation dialog', async () => {
    render(SessionForm);

    // Make the form dirty
    const durationInput = screen.getByLabelText('Duration (hours)');
    await fireEvent.input(durationInput, { target: { value: '3' } });

    // Press ESC key to show confirmation
    await fireEvent.keyDown(window, { key: 'Escape' });

    // Click "Continue" button in confirmation dialog
    const continueButton = screen.getByText('Continue');
    await fireEvent.click(continueButton);

    // Should dispatch cancel
    expect(mockDispatch).toHaveBeenCalledWith('cancel');
  });

  it('does not show confirmation dialog when form is pristine', async () => {
    render(SessionForm);

    // Click cancel button without making changes
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);

    // Should dispatch cancel immediately
    expect(mockDispatch).toHaveBeenCalledWith('cancel');

    // Confirmation dialog should not be visible
    expect(screen.queryByText('Unsaved Changes')).not.toBeInTheDocument();
  });

  it('does not trigger ESC when confirmation dialog is open', async () => {
    render(SessionForm);

    // Make the form dirty
    const buyInInput = screen.getByLabelText('Buy-in ($)');
    await fireEvent.input(buyInInput, { target: { value: '100' } });

    // Press ESC key to show confirmation
    await fireEvent.keyDown(window, { key: 'Escape' });

    // Confirmation dialog should be visible
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();

    // Clear mock
    mockDispatch.mockClear();

    // Press ESC key again while confirmation is showing
    await fireEvent.keyDown(window, { key: 'Escape' });

    // Should not dispatch cancel
    expect(mockDispatch).not.toHaveBeenCalled();
  });

  it('tracks dirty state correctly for all form fields', async () => {
    render(SessionForm, { props: { session: mockSessions[0] } });

    // Initially pristine - should close immediately
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);
    expect(mockDispatch).toHaveBeenCalledWith('cancel');
    mockDispatch.mockClear();

    // Change each field and verify dirty state
    const dateInput = screen.getByLabelText('Session Date');
    await fireEvent.input(dateInput, { target: { value: '2024-03-01' } });
    await fireEvent.click(cancelButton);
    expect(screen.getByText('Unsaved Changes')).toBeInTheDocument();

    // Close confirmation by clicking the primary Cancel button (with data-default)
    const cancelButtons = screen.getAllByText('Cancel');
    const confirmDialogCancelButton = cancelButtons.find(btn => btn.hasAttribute('data-default'));
    await fireEvent.click(confirmDialogCancelButton!);
    expect(screen.queryByText('Unsaved Changes')).not.toBeInTheDocument();
  });
});
