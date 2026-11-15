import { Grid, Group, Progress, Stack, Table, Text } from '@mantine/core';
import { StatusBadge } from '../../components/StatusBadge';
import { fmtDateTime } from '../../lib/dates';
import { formatBounds, formatValue, statusColor } from '../../lib/status';
import type { MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';
import { MetricChart } from './MetricChart';
import { SnapshotBadge } from './MetricChartWidget';

/**
 * Default widget for metrics with a derived score (e.g. sleep): trend of the
 * overall score plus a detailed report of the latest reading.
 */
export function ScoreWidget({ ctx, metric, snapshot }: MetricWidgetProps) {
  const score = metric.components.find((c) => c.derived)!;
  const parts = metric.components.filter((c) => !c.derived);
  const max = parts.reduce((sum, c) => sum + (c.scoreMax ?? 0), 0);
  const value = snapshot?.values[score.key];
  const status = snapshot?.componentStatuses[score.key] ?? 'unknown';

  return (
    <WidgetCard title={metric.name} subtitle={metric.description} right={<SnapshotBadge snapshot={snapshot} />}>
      <Grid gutter="lg">
        <Grid.Col span={{ base: 12, md: 7 }}>
          <Text size="sm" fw={600} mb={4}>
            {score.label} w czasie
          </Text>
          <MetricChart astronautId={ctx.astronautId} metric={metric} range={ctx.range} components={[score.key]} />
        </Grid.Col>
        <Grid.Col span={{ base: 12, md: 5 }}>
          <Stack gap={6}>
            <Group justify="space-between" align="baseline">
              <Text size="sm" fw={600}>
                Ostatnia noc
              </Text>
              {snapshot?.recordedAt && (
                <Text size="xs" c="dimmed">
                  {fmtDateTime(snapshot.recordedAt)}
                </Text>
              )}
            </Group>
            <Group align="baseline" gap={6}>
              <Text fz={32} fw={800} lh={1} c={`${statusColor[status]}.5`}>
                {value ?? '—'}
              </Text>
              <Text c="dimmed">/ {max} pkt</Text>
            </Group>
            <Progress value={value ? (value / max) * 100 : 0} color={statusColor[status]} size="sm" />
            <Table fz="sm" verticalSpacing={4} mt={4}>
              <Table.Tbody>
                {parts.map((c) => (
                  <Table.Tr key={c.key}>
                    <Table.Td>{c.label}</Table.Td>
                    <Table.Td fw={600}>{formatValue(snapshot?.values[c.key], c)}</Table.Td>
                    <Table.Td c="dimmed" visibleFrom="sm">
                      {formatBounds(c.reference.normal, c.unit)}
                    </Table.Td>
                    <Table.Td>
                      <StatusBadge status={snapshot?.componentStatuses[c.key] ?? 'unknown'} size="xs" />
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </Stack>
        </Grid.Col>
      </Grid>
    </WidgetCard>
  );
}
