import { Badge, Group, Table, Text } from '@mantine/core';
import { useSeries } from '../../api/hooks';
import { IconCalendarDue } from '@tabler/icons-react';
import { StatusBadge } from '../../components/StatusBadge';
import { dayjs, fmtDate } from '../../lib/dates';
import { formatBounds, formatValue } from '../../lib/status';
import type { MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';
import { MetricChart } from './MetricChart';
import { SnapshotBadge } from './MetricChartWidget';

/**
 * Surface sampling campaigns: results of the latest swab per location, the
 * sampling schedule (next due / overdue), the most recent campaigns and the
 * history chart.
 */
const RECENT = 5;

export function SamplingWidget({ ctx, metric, snapshot }: MetricWidgetProps) {
  const history = useSeries(ctx.astronautId, metric.code, {});
  const recent = [...(history.data?.points ?? [])].reverse().slice(0, RECENT);
  const last = snapshot?.recordedAt ? dayjs(snapshot.recordedAt) : null;
  const next = last?.add(metric.samplingIntervalHours, 'hour');
  const overdue = !!next && next.isBefore(dayjs());

  return (
    <WidgetCard
      title={metric.name}
      subtitle={metric.description}
      right={<SnapshotBadge snapshot={snapshot} stationWide={metric.scope === 'station'} />}
    >
      <Group gap="xs" mb="xs">
        <IconCalendarDue size={18} />
        <Text size="sm">
          Ostatnie pobranie: <b>{last ? fmtDate(last.toISOString()) : '—'}</b>
        </Text>
        {next && (
          <Badge color={overdue ? 'blue' : 'gray'} variant="light">
            {overdue ? `Zaległe od ${fmtDate(next.toISOString())}` : `Następne: ${fmtDate(next.toISOString())}`}
          </Badge>
        )}
      </Group>
      <Table fz="sm" verticalSpacing={4} mb="sm">
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Miejsce pobrania</Table.Th>
            <Table.Th>Wynik</Table.Th>
            <Table.Th visibleFrom="sm">Norma</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {metric.components.map((c) => (
            <Table.Tr key={c.key}>
              <Table.Td>{c.label}</Table.Td>
              <Table.Td fw={600}>{formatValue(snapshot?.values[c.key], c)}</Table.Td>
              <Table.Td c="dimmed" visibleFrom="sm">
                {formatBounds(c.reference.normal, c.unit)}
              </Table.Td>
              <Table.Td>
                <StatusBadge status={snapshot?.componentStatuses[c.key] ?? 'unknown'} size="sm" />
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      {recent.length > 1 && (
        <>
          <Text size="sm" fw={600} mb={4}>
            Ostatnie pobrania
          </Text>
          <Table.ScrollContainer minWidth={420}>
            <Table fz="xs" verticalSpacing={2} mb="sm" withColumnBorders={false}>
              <Table.Thead>
                <Table.Tr>
                  <Table.Th>Data</Table.Th>
                  {metric.components.map((c) => (
                    <Table.Th key={c.key}>{c.label}</Table.Th>
                  ))}
                  <Table.Th />
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {recent.map((p) => (
                  <Table.Tr key={p.t}>
                    <Table.Td>{fmtDate(p.t)}</Table.Td>
                    {metric.components.map((c) => (
                      <Table.Td key={c.key} c={p.statuses[c.key] === 'normal' ? undefined : 'orange'}>
                        {p.values[c.key]?.toFixed(c.decimals) ?? '—'}
                      </Table.Td>
                    ))}
                    <Table.Td>
                      <StatusBadge status={p.status} size="xs" />
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </Table.ScrollContainer>
        </>
      )}
      <MetricChart astronautId={ctx.astronautId} metric={metric} range={ctx.range} height={180} />
    </WidgetCard>
  );
}
