import type { ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import type { Role } from '../api/types';
import { useAuth } from './AuthContext';

/** UI-level guard only — the backend enforces the same rules independently. */
export function RequireRole({ role, children }: { role?: Role; children: ReactNode }) {
  const { user } = useAuth();
  const location = useLocation();
  if (!user) return <Navigate to="/login" replace state={{ from: location.pathname }} />;
  if (role && user.role !== role) return <Navigate to={homeFor(user.role)} replace />;
  return <>{children}</>;
}

export function homeFor(role: Role) {
  return role === 'supervisor' ? '/crew' : '/me';
}
