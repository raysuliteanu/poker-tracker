import { vi } from 'vitest';
import type { User, PokerSession, AuthResponse } from '../lib/api';

// Test fixtures
export const mockUser: User = {
  id: 'user-123',
  email: 'test@example.com',
  username: 'testuser',
  cookie_consent: true,
  cookie_consent_date: '2024-01-01T00:00:00Z',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

export const mockSessions: PokerSession[] = [
  {
    id: 'session-1',
    user_id: 'user-123',
    session_date: '2024-01-15',
    duration_minutes: 120,
    buy_in_amount: '100.00',
    rebuy_amount: '0.00',
    cash_out_amount: '150.00',
    notes: 'Good session',
    created_at: '2024-01-15T00:00:00Z',
    updated_at: '2024-01-15T00:00:00Z',
    profit: 50,
  },
  {
    id: 'session-2',
    user_id: 'user-123',
    session_date: '2024-01-20',
    duration_minutes: 180,
    buy_in_amount: '200.00',
    rebuy_amount: '50.00',
    cash_out_amount: '180.00',
    notes: 'Tough table',
    created_at: '2024-01-20T00:00:00Z',
    updated_at: '2024-01-20T00:00:00Z',
    profit: -70,
  },
  {
    id: 'session-3',
    user_id: 'user-123',
    session_date: '2024-01-25',
    duration_minutes: 90,
    buy_in_amount: '100.00',
    rebuy_amount: '0.00',
    cash_out_amount: '220.00',
    notes: null,
    created_at: '2024-01-25T00:00:00Z',
    updated_at: '2024-01-25T00:00:00Z',
    profit: 120,
  },
];

export const mockAuthResponse: AuthResponse = {
  token: 'mock-jwt-token',
  user: mockUser,
};

// API mock factory
export function createApiMock() {
  return {
    auth: {
      register: vi.fn(),
      login: vi.fn(),
      getMe: vi.fn(),
      updateCookieConsent: vi.fn(),
      changePassword: vi.fn(),
    },
    sessions: {
      create: vi.fn(),
      getAll: vi.fn(),
      getOne: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    },
  };
}

// Fetch mock helper
export function mockFetch(response: unknown, ok = true) {
  return vi.fn().mockResolvedValue({
    ok,
    json: () => Promise.resolve(response),
  });
}

// Create a successful API response
export function successResponse<T>(data: T) {
  return { data };
}

// Create an error API response
export function errorResponse(error: string) {
  return { error };
}
