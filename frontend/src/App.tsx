import { Navigate, Route, Routes } from 'react-router-dom';
import { useAuth } from './auth/AuthContext';
import { RequireRole, homeFor } from './auth/RequireRole';
import { AppLayout } from './components/AppLayout';
import { LoginPage } from './pages/LoginPage';
import { MyProfilePage } from './pages/MyProfilePage';
import { SupervisorPage } from './pages/SupervisorPage';

export function App() {
  const { user } = useAuth();
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        element={
          <RequireRole>
            <AppLayout />
          </RequireRole>
        }
      >
        <Route
          path="/me"
          element={
            <RequireRole role="astronaut">
              <MyProfilePage />
            </RequireRole>
          }
        />
        <Route
          path="/crew/:astronautId?"
          element={
            <RequireRole role="supervisor">
              <SupervisorPage />
            </RequireRole>
          }
        />
      </Route>
      <Route path="*" element={<Navigate to={user ? homeFor(user.role) : '/login'} replace />} />
    </Routes>
  );
}
