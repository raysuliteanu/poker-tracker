import { writable } from 'svelte/store';
import type { User } from '../lib/api';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    user: null,
    token: localStorage.getItem('token'),
    isAuthenticated: !!localStorage.getItem('token'),
  });

  return {
    subscribe,
    login: (token: string, user: User) => {
      localStorage.setItem('token', token);
      set({ user, token, isAuthenticated: true });
    },
    logout: () => {
      localStorage.removeItem('token');
      set({ user: null, token: null, isAuthenticated: false });
    },
    updateUser: (user: User) => {
      update((state) => ({ ...state, user }));
    },
  };
}

export const authStore = createAuthStore();
