import { Alert } from '@mantine/core';
import { useAuth } from '../auth/AuthContext';
import { HealthProfile } from '../components/HealthProfile';

/** Astronaut view: only the logged-in astronaut's own health profile. */
export function MyProfilePage() {
  const { user } = useAuth();
  if (!user?.astronautId) return <Alert color="red">To konto nie jest powiązane z astronautą.</Alert>;
  return <HealthProfile astronautId={user.astronautId} />;
}
