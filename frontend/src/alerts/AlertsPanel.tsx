import { Stack, Text, Title } from '@mantine/core';
import type { Alert, AlertSeverity, MetricGroup } from '../api/types';
import { severityLabel } from '../lib/status';
import { AlertItem } from './AlertItem';

const ORDER: AlertSeverity[] = ['critical', 'warning', 'notice'];

/** Dedicated tab with every alert, grouped by severity, with all findings. */
export function AlertsPanel({
  alerts,
  groups,
  onOpenAlert,
}: {
  alerts: Alert[];
  groups: MetricGroup[];
  onOpenAlert: (alert: Alert) => void;
}) {
  const groupName = (code: string) => groups.find((g) => g.code === code)?.name ?? code;
  if (alerts.length === 0) {
    return (
      <Text c="dimmed" ta="center" py="xl">
        Brak alertów — wszystkie parametry są w normie.
      </Text>
    );
  }
  return (
    <Stack gap="lg">
      {ORDER.map((severity) => {
        const items = alerts.filter((a) => a.severity === severity);
        if (items.length === 0) return null;
        return (
          <Stack key={severity} gap="xs">
            <Title order={5}>
              {severityLabel[severity]} ({items.length})
            </Title>
            {items.map((a) => (
              <AlertItem key={a.id} alert={a} groupName={groupName(a.group)} onOpen={onOpenAlert} detailed />
            ))}
          </Stack>
        );
      })}
    </Stack>
  );
}
