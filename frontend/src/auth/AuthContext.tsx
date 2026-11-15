import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { api, sessionStore, setToken, setUnauthorizedHandler } from '../api/client';
import type { LoginResponse, User } from '../api/types';

interface Session {
  token: string;
  expiresAt: string;
  user: User;
}

interface AuthContextValue {
  user: User | null;
  login: (username: string, password: string) => Promise<User>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function restore(): Session | null {
  const s = sessionStore.load<Session>();
  if (!s || new Date(s.expiresAt).getTime() <= Date.now()) return null;
  return s;
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const [session, setSession] = useState<Session | null>(() => {
    const s = restore();
    setToken(s?.token ?? null);
    return s;
  });

  const logout = useCallback(() => {
    setToken(null);
    sessionStore.clear();
    setSession(null);
    queryClient.clear();
  }, [queryClient]);

  setUnauthorizedHandler(logout);

  const login = useCallback(async (username: string, password: string) => {
    const res = await api<LoginResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
    const s: Session = { token: res.token, expiresAt: res.expiresAt, user: res.user };
    setToken(s.token);
    sessionStore.save(s);
    setSession(s);
    return s.user;
  }, []);

  const value = useMemo(
    () => ({ user: session?.user ?? null, login, logout }),
    [session, login, logout],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside AuthProvider');
  return ctx;
}
