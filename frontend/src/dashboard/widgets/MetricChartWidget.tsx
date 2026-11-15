import { Badge, Group, SimpleGrid, Text, Tooltip } from '@mantine/core';
import { IconClockExclamation, IconSatellite } from '@tabler/icons-react';
import { StatusBadge } from '../../components/StatusBadge';
import { fmtDateTime } from '../../lib/dates';
import { formatBounds, formatValue, statusColor } from '../../lib/status';
import type { MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';
import { MetricChart } from './MetricChart';

/** Default widget used for every metric without a custom one. */
export function MetricChartWidget({ ctx, metric, snapshot }: MetricWidgetProps) {
  return (
    <WidgetCard title={metric.name} subtitle={metric.description} right={<SnapshotBadge snapshot={snapshot} stationWide={metric.scope === 'station'} />}>
      <SimpleGrid cols={{ base: 2, sm: Math.min(metric.components.length, 3) }} mb="sm">
        {metric.components.filter((c) => !c.derived).map((c) => {
          const status = snapshot?.componentStatuses[c.key] ?? 'unknown';
          return (
            <Tooltip key={c.key} label={`Norma: ${formatBounds(c.reference.normal, c.unit)}`}>
              <div>
                <Text size="xs" c="dimmed">
                  {c.label}
                </Text>
                <Text fw={700} size="lg" c={`${statusColor[status]}.5`}>
                  {formatValue(snapshot?.values[c.key], c)}
                </Text>
              </div>
            </Tooltip>
          );
        })}
      </SimpleGrid>
      <MetricChart astronautId={ctx.astronautId} metric={metric} range={ctx.range} />
    </WidgetCard>
  );
}

export function SnapshotBadge({
  snapshot,
  stationWide = false,
}: {
  snapshot: MetricWidgetProps['snapshot'];
  stationWide?: boolean;
}) {
  return (
    <Group gap={6} wrap="nowrap">
      {stationWide && (
        <Tooltip label="Pomiar środowiska stacji — wspólny dla całej załogi">
          <Badge variant="outline" color="gray" leftSection={<IconSatellite size={12} />} style={{ flexShrink: 0 }}>
            Cała stacja
          </Badge>
        </Tooltip>
      )}
      {snapshot?.stale && (
        <Tooltip label="Brak aktualnego pomiaru (przekroczony interwał)">
          <IconClockExclamation size={18} color="var(--mantine-color-orange-5)" />
        </Tooltip>
      )}
      <Tooltip label={snapshot?.recordedAt ? `Pomiar: ${fmtDateTime(snapshot.recordedAt)}` : 'Brak pomiarów'}>
        <span>
          <StatusBadge status={snapshot?.status ?? 'unknown'} />
        </span>
      </Tooltip>
    </Group>
  );
}
