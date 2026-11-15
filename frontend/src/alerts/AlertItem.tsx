import { Badge, Group, Paper, Stack, Table, Text, UnstyledButton } from '@mantine/core';
import { IconChevronRight } from '@tabler/icons-react';
import type { Alert } from '../api/types';
import { StatusBadge } from '../components/StatusBadge';
import { fromNow } from '../lib/dates';
import { formatBounds, severityColor, severityLabel } from '../lib/status';
import { SeverityIcon } from './SeverityIcon';

/**
 * One alert. The whole row is a link to the full report of the parameter
 * (its tab, scrolled to its chart / lab table / body map).
 *
 * `variant="banner"` is the compact, tinted row shown at the top of a
 * profile; `"row"` is the row used in the full alerts list. Both always show
 * the severity (icon + label), the content and the time of the measurement.
 */
export function AlertItem({
  alert,
  groupName,
  onOpen,
  detailed = false,
  variant = 'row',
}: {
  alert: Alert;
  groupName: string;
  onOpen: (alert: Alert) => void;
  detailed?: boolean;
  variant?: 'row' | 'banner';
}) {
  const banner = variant === 'banner';
  const color = severityColor[alert.severity];
  const time = alert.recordedAt ? `pomiar ${fromNow(alert.recordedAt)}` : null;
  return (
    <Paper
      withBorder
      radius="md"
      bg={banner ? `var(--mantine-color-${color}-light)` : undefined}
      style={{ borderLeft: `${banner ? 4 : 6}px solid var(--mantine-color-${color}-6)` }}
      data-testid={banner ? 'alert-banner' : 'alert-row'}
    >
      <UnstyledButton
        onClick={() => onOpen(alert)}
        py={banner ? 6 : 'sm'}
        px="sm"
        w="100%"
        aria-label={`${alert.title}: otwórz raport`}
      >
        <Group wrap="nowrap" gap="sm">
          <SeverityIcon severity={alert.severity} size={banner ? 26 : 34} />
          <Stack gap={banner ? 0 : 2} style={{ flex: 1, minWidth: 0 }}>
            <Group gap={6} wrap="nowrap">
              <Badge
                color={color}
                variant={alert.severity === 'notice' ? 'light' : 'filled'}
                size={banner ? 'xs' : 'sm'}
                style={{ flexShrink: 0 }}
              >
                {severityLabel[alert.severity]}
              </Badge>
              <Text fw={banner ? 700 : 800} size="sm" truncate>
                {alert.title}
              </Text>
            </Group>
            <Text size={banner ? 'xs' : 'sm'} c={banner ? undefined : 'dimmed'} lineClamp={banner ? 2 : undefined}>
              {alert.message}
            </Text>
            {/* On narrow screens the side column is hidden; keep the time visible. */}
            {time && (
              <Text size="xs" c="dimmed" hiddenFrom="sm">
                {time}
              </Text>
            )}
          </Stack>
          <Stack gap={0} align="flex-end" visibleFrom="sm" style={{ flexShrink: 0 }}>
            <Text size="xs" fw={600} c={`${color}.5`}>
              {banner ? groupName : `Zobacz raport: ${groupName}`}
            </Text>
            {time && (
              <Text size="xs" c="dimmed">
                {time}
              </Text>
            )}
          </Stack>
          <IconChevronRight size={banner ? 16 : 18} style={{ flexShrink: 0 }} />
        </Group>
      </UnstyledButton>
      {detailed && alert.findings.length > 0 && (
        <Table fz="sm" verticalSpacing={4} px="sm" pb="xs" withRowBorders={false} layout="fixed">
          <Table.Tbody>
            {alert.findings.map((f) => (
              <Table.Tr key={f.component}>
                <Table.Td pl={58} w="40%">
                  {f.label}
                </Table.Td>
                <Table.Td fw={600}>
                  {f.value} {f.unit}
                </Table.Td>
                <Table.Td c="dimmed">norma {formatBounds(f.normal, f.unit)}</Table.Td>
                <Table.Td>
                  <StatusBadge status={f.status} size="sm" />
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      )}
    </Paper>
  );
}
