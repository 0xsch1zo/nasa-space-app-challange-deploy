import { useState } from 'react';
import {
  Alert,
  Anchor,
  Button,
  Center,
  Code,
  Paper,
  PasswordInput,
  Stack,
  Table,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { IconSatellite } from '@tabler/icons-react';
import { Navigate, useNavigate } from 'react-router-dom';
import { useAuth } from '../auth/AuthContext';
import { homeFor } from '../auth/RequireRole';

const DEMO_ACCOUNTS = [
  ['lekarz.aurora', 'Nadzór — Aurora'],
  ['lekarz.selene', 'Nadzór — Selene'],
  ['anna.kowalska', 'Astronautka — Aurora'],
  ['marcus.reyes', 'Astronauta — Aurora'],
  ['omar.haddad', 'Astronauta — Selene'],
];

export function LoginPage() {
  const { user, login } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  if (user) return <Navigate to={homeFor(user.role)} replace />;

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const u = await login(username, password);
      navigate(homeFor(u.role), { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Logowanie nie powiodło się');
    } finally {
      setLoading(false);
    }
  }

  return (
    <Center mih="100vh" p="md">
      <Stack w="100%" maw={420}>
        <Stack align="center" gap={4}>
          <IconSatellite size={42} />
          <Title order={2}>AstroHealth</Title>
          <Text c="dimmed" size="sm">
            Monitoring zdrowia załogi stacji kosmicznej
          </Text>
        </Stack>
        <Paper withBorder p="lg" component="form" onSubmit={submit}>
          <Stack>
            {error && <Alert color="red">{error}</Alert>}
            <TextInput
              label="Login"
              value={username}
              onChange={(e) => setUsername(e.currentTarget.value)}
              autoComplete="username"
              required
            />
            <PasswordInput
              label="Hasło"
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
              autoComplete="current-password"
              required
            />
            <Button type="submit" loading={loading} fullWidth>
              Zaloguj
            </Button>
          </Stack>
        </Paper>
        <Paper withBorder p="md">
          <Text size="sm" fw={600} mb="xs">
            Konta demonstracyjne (hasło <Code>orbita2026</Code>)
          </Text>
          <Table fz="sm" verticalSpacing={4}>
            <Table.Tbody>
              {DEMO_ACCOUNTS.map(([login, desc]) => (
                <Table.Tr key={login}>
                  <Table.Td>
                    <Anchor
                      component="button"
                      type="button"
                      onClick={() => {
                        setUsername(login);
                        setPassword('orbita2026');
                      }}
                    >
                      {login}
                    </Anchor>
                  </Table.Td>
                  <Table.Td c="dimmed">{desc}</Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Paper>
      </Stack>
    </Center>
  );
}
