import { Group, Text } from '@mantine/core';
import { fmtDateTime } from '../../lib/dates';
import { statusColor } from '../../lib/status';
import type { MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';
import { MetricChart } from './MetricChart';
import { SnapshotBadge } from './MetricChartWidget';

/**
 * Example of a metric-specific widget registered in `metricWidgetOverrides`:
 * blood pressure is conventionally read as "systolic/diastolic", plus pulse
 * pressure. It reuses the generic chart, so only the header is bespoke.
 */
export function BloodPressureWidget({ ctx, metric, snapshot }: MetricWidgetProps) {
  const sys = snapshot?.values.systolic;
  const dia = snapshot?.values.diastolic;
  const color = statusColor[snapshot?.status ?? 'unknown'];

  return (
    <WidgetCard title={metric.name} subtitle={metric.description} right={<SnapshotBadge snapshot={snapshot} />}>
      <Group align="baseline" gap="xs" mb="sm">
        <Text fz={34} fw={800} c={`${color}.5`} lh={1}>
          {sys !== undefined && dia !== undefined ? `${sys.toFixed(0)}/${dia.toFixed(0)}` : '—'}
        </Text>
        <Text c="dimmed">mmHg</Text>
        {sys !== undefined && dia !== undefined && (
          <Text size="sm" c="dimmed" ml="md">
            ciśnienie tętna: {(sys - dia).toFixed(0)} mmHg
          </Text>
        )}
        {snapshot?.recordedAt && (
          <Text size="xs" c="dimmed" ml="auto">
            {fmtDateTime(snapshot.recordedAt)}
          </Text>
        )}
      </Group>
      <MetricChart astronautId={ctx.astronautId} metric={metric} range={ctx.range} />
    </WidgetCard>
  );
}
