const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

interface ApiResponse<T> {
  data?: T;
  error?: string;
}

async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const token = localStorage.getItem('token');

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  try {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    });

    const data = await response.json();

    if (!response.ok) {
      return { error: data.error || 'An error occurred' };
    }

    return { data };
  } catch (error) {
    return { error: 'Network error. Please try again.' };
  }
}

export interface User {
  id: string;
  email: string;
  username: string;
  cookie_consent: boolean;
  cookie_consent_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface PokerSession {
  id: string;
  user_id: string;
  session_date: string;
  duration_minutes: number;
  buy_in_amount: string;
  rebuy_amount: string;
  cash_out_amount: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
  profit?: number;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export const api = {
  auth: {
    register: (email: string, username: string, password: string) =>
      apiRequest<AuthResponse>('/auth/register', {
        method: 'POST',
        body: JSON.stringify({ email, username, password }),
      }),

    login: (email: string, password: string) =>
      apiRequest<AuthResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email, password }),
      }),

    getMe: () => apiRequest<User>('/auth/me'),

    updateCookieConsent: (cookie_consent: boolean) =>
      apiRequest<User>('/auth/cookie-consent', {
        method: 'PUT',
        body: JSON.stringify({ cookie_consent }),
      }),

    changePassword: (old_password: string, new_password: string) =>
      apiRequest<{ message: string }>('/auth/change-password', {
        method: 'POST',
        body: JSON.stringify({ old_password, new_password }),
      }),
  },

  sessions: {
    create: (session: {
      session_date: string;
      duration_minutes: number;
      buy_in_amount: number;
      rebuy_amount?: number;
      cash_out_amount: number;
      notes?: string;
    }) =>
      apiRequest<PokerSession>('/sessions', {
        method: 'POST',
        body: JSON.stringify(session),
      }),

    getAll: () => apiRequest<PokerSession[]>('/sessions'),

    getOne: (id: string) => apiRequest<PokerSession>(`/sessions/${id}`),

    update: (
      id: string,
      updates: {
        session_date?: string;
        duration_minutes?: number;
        buy_in_amount?: number;
        rebuy_amount?: number;
        cash_out_amount?: number;
        notes?: string;
      }
    ) =>
      apiRequest<PokerSession>(`/sessions/${id}`, {
        method: 'PUT',
        body: JSON.stringify(updates),
      }),

    delete: (id: string) =>
      apiRequest<{ message: string }>(`/sessions/${id}`, {
        method: 'DELETE',
      }),
  },
};
