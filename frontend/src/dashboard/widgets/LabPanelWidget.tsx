import { useState } from 'react';
import { Badge, Group, Select, Table, Text } from '@mantine/core';
import { IconArrowDownRight, IconArrowUpRight } from '@tabler/icons-react';
import { useSeries } from '../../api/hooks';
import { QueryState } from '../../components/QueryState';
import { StatusBadge } from '../../components/StatusBadge';
import { fmtDate } from '../../lib/dates';
import { formatBounds } from '../../lib/status';
import type { MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';

/**
 * Default widget for `table` metrics (lab panels): results of one exam with
 * the status of every result, a picker for previous exams and the change
 * versus the exam before. Statuses come from the backend for every exam.
 */
export function LabPanelWidget({ ctx, metric }: MetricWidgetProps) {
  const series = useSeries(ctx.astronautId, metric.code, {});
  const exams = [...(series.data?.points ?? [])].reverse(); // newest first
  const [selectedT, setSelectedT] = useState<string | null>(null);
  const idx = Math.max(0, exams.findIndex((e) => e.t === selectedT));
  const exam = exams[idx];
  const previous = exams[idx + 1];
  const outOfRange = exam ? metric.components.filter((c) => (exam.statuses[c.key] ?? 'normal') !== 'normal') : [];
  const highlight = ctx.focus?.metric === metric.code ? ctx.focus.component : undefined;

  return (
    <WidgetCard
      title={metric.name}
      subtitle={metric.description}
      right={
        exam && (
          <Badge color={outOfRange.length ? (exam.status === 'critical' ? 'red' : 'yellow') : 'teal'} variant="light" style={{ flexShrink: 0 }}>
            {outOfRange.length ? `${outOfRange.length} poza normą` : 'Wszystko w normie'}
          </Badge>
        )
      }
    >
      <QueryState isLoading={series.isLoading} error={series.error}>
        {!exam ? (
          <Text c="dimmed" size="sm">
            Brak wyników badań.
          </Text>
        ) : (
          <>
            <Group justify="space-between" mb="xs">
              <Select
                size="xs"
                w={190}
                label="Badanie z dnia"
                allowDeselect={false}
                value={exam.t}
                onChange={setSelectedT}
                data={exams.map((e, i) => ({
                  value: e.t,
                  label: `${fmtDate(e.t)}${i === 0 ? ' (ostatnie)' : ''}`,
                }))}
              />
              <Text size="xs" c="dimmed">
                {exams.length} {exams.length === 1 ? 'badanie' : 'badań'} w historii
              </Text>
            </Group>
            <Table.ScrollContainer minWidth={560}>
              <Table verticalSpacing={6} fz="sm">
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Oznaczenie</Table.Th>
                    <Table.Th>Wynik</Table.Th>
                    <Table.Th>Norma</Table.Th>
                    <Table.Th>Poprzednio</Table.Th>
                    <Table.Th />
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {metric.components.map((c) => {
                    const v = exam.values[c.key];
                    const p = previous?.values[c.key];
                    const status = exam.statuses[c.key] ?? 'unknown';
                    return (
                      <Table.Tr
                        key={c.key}
                        bg={c.key === highlight ? 'var(--mantine-primary-color-light)' : undefined}
                      >
                        <Table.Td>{c.label}</Table.Td>
                        <Table.Td fw={600}>
                          {v === undefined ? '—' : `${v.toFixed(c.decimals)} ${c.unit}`}
                        </Table.Td>
                        <Table.Td c="dimmed">{formatBounds(c.reference.normal, c.unit)}</Table.Td>
                        <Table.Td c="dimmed">
                          {p === undefined || v === undefined ? (
                            '—'
                          ) : (
                            <Group gap={2} wrap="nowrap">
                              {p.toFixed(c.decimals)}
                              {v > p ? <IconArrowUpRight size={14} /> : v < p ? <IconArrowDownRight size={14} /> : null}
                            </Group>
                          )}
                        </Table.Td>
                        <Table.Td miw={110}>
                          <StatusBadge status={status} size="sm" />
                        </Table.Td>
                      </Table.Tr>
                    );
                  })}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
          </>
        )}
      </QueryState>
    </WidgetCard>
  );
}
