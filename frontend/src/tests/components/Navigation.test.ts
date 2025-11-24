import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/svelte';
import { get } from 'svelte/store';
import Navigation from '../../components/Navigation.svelte';
import { authStore } from '../../stores/auth';
import { themeStore } from '../../stores/theme';
import { mockUser } from '../mocks';

describe('Navigation', () => {
  beforeEach(() => {
    cleanup();
    authStore.logout(); // Reset auth state
    themeStore.set('light'); // Reset theme
    vi.clearAllMocks();
  });

  it('renders nothing when user is not authenticated', () => {
    // Ensure not authenticated
    expect(get(authStore).isAuthenticated).toBe(false);

    const { container } = render(Navigation);

    // Should not render navbar
    expect(container.querySelector('.navbar')).toBeNull();
  });

  it('renders navbar when user is authenticated', () => {
    authStore.login('test-token', mockUser);

    render(Navigation);

    expect(screen.getByText('Poker Tracker')).toBeInTheDocument();
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Charts')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('Logout')).toBeInTheDocument();
  });

  it('renders navigation links with correct hrefs', () => {
    authStore.login('test-token', mockUser);

    render(Navigation);

    const dashboardLink = screen.getByText('Dashboard').closest('a');
    const chartsLink = screen.getByText('Charts').closest('a');
    const settingsLink = screen.getByText('Settings').closest('a');

    expect(dashboardLink).toHaveAttribute('href', '#/');
    expect(chartsLink).toHaveAttribute('href', '#/charts');
    expect(settingsLink).toHaveAttribute('href', '#/settings');
  });

  it('displays moon icon for light theme', () => {
    authStore.login('test-token', mockUser);
    themeStore.set('light');

    render(Navigation);

    const themeButton = screen.getByTitle('Toggle theme');
    expect(themeButton.textContent?.trim()).toContain('🌙');
  });

  it('displays sun icon for dark theme', () => {
    authStore.login('test-token', mockUser);
    themeStore.set('dark');

    render(Navigation);

    const themeButton = screen.getByTitle('Toggle theme');
    expect(themeButton.textContent?.trim()).toContain('☀️');
  });

  it('toggles theme when theme button is clicked', async () => {
    authStore.login('test-token', mockUser);

    render(Navigation);

    const initialTheme = get(themeStore);
    expect(initialTheme).toBe('light');

    const themeButton = screen.getByTitle('Toggle theme');
    await fireEvent.click(themeButton);

    expect(get(themeStore)).toBe('dark');
  });

  it('calls logout and reloads page when logout button is clicked', async () => {
    authStore.login('test-token', mockUser);

    render(Navigation);

    const logoutButton = screen.getByText('Logout');
    await fireEvent.click(logoutButton);

    // Check that auth state was cleared
    const state = get(authStore);
    expect(state.isAuthenticated).toBe(false);
    expect(state.token).toBeNull();
    expect(state.user).toBeNull();

    // Check that reload was called
    expect(window.location.reload).toHaveBeenCalled();
  });
});
