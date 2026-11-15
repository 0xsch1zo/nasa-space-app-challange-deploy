import { Alert, Center, Loader } from '@mantine/core';
import { IconAlertTriangle } from '@tabler/icons-react';
import type { ReactNode } from 'react';

/** Uniform loading / error rendering for react-query results. */
export function QueryState({
  isLoading,
  error,
  children,
  minHeight = 120,
}: {
  isLoading: boolean;
  error: unknown;
  children: ReactNode;
  minHeight?: number;
}) {
  if (isLoading)
    return (
      <Center mih={minHeight}>
        <Loader size="sm" />
      </Center>
    );
  if (error)
    return (
      <Alert color="red" icon={<IconAlertTriangle size={18} />} title="Nie udało się pobrać danych">
        {error instanceof Error ? error.message : String(error)}
      </Alert>
    );
  return <>{children}</>;
}
