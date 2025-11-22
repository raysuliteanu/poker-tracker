import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { mockUser } from '../mocks';

describe('authStore', () => {
  beforeEach(async () => {
    // Reset modules to get fresh store instance
    vi.resetModules();
    localStorage.clear();
  });

  it('initializes with no authentication when no token in localStorage', async () => {
    const { authStore } = await import('../../stores/auth');
    const state = get(authStore);

    expect(state.isAuthenticated).toBe(false);
    expect(state.token).toBeNull();
    expect(state.user).toBeNull();
  });

  it('initializes with authentication when token exists in localStorage', async () => {
    localStorage.setItem('token', 'existing-token');

    const { authStore } = await import('../../stores/auth');
    const state = get(authStore);

    expect(state.isAuthenticated).toBe(true);
    expect(state.token).toBe('existing-token');
    expect(state.user).toBeNull(); // User is not loaded from localStorage
  });

  it('login() sets token, user, and isAuthenticated', async () => {
    const { authStore } = await import('../../stores/auth');

    authStore.login('new-token', mockUser);
    const state = get(authStore);

    expect(state.isAuthenticated).toBe(true);
    expect(state.token).toBe('new-token');
    expect(state.user).toEqual(mockUser);
    expect(localStorage.getItem('token')).toBe('new-token');
  });

  it('logout() clears state and removes token from localStorage', async () => {
    localStorage.setItem('token', 'existing-token');

    const { authStore } = await import('../../stores/auth');
    authStore.login('new-token', mockUser);

    authStore.logout();
    const state = get(authStore);

    expect(state.isAuthenticated).toBe(false);
    expect(state.token).toBeNull();
    expect(state.user).toBeNull();
    expect(localStorage.getItem('token')).toBeNull();
  });

  it('updateUser() updates the user while preserving other state', async () => {
    const { authStore } = await import('../../stores/auth');
    authStore.login('test-token', mockUser);

    const updatedUser = { ...mockUser, username: 'newusername' };
    authStore.updateUser(updatedUser);
    const state = get(authStore);

    expect(state.user?.username).toBe('newusername');
    expect(state.token).toBe('test-token');
    expect(state.isAuthenticated).toBe(true);
  });
});
