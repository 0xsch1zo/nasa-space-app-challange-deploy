import { useEffect, useMemo, useState } from 'react';
import { Grid, Group, SegmentedControl, Text, Title } from '@mantine/core';
import type { Alert, MetricDefinition, MetricGroup, Overview } from '../api/types';
import { AlertsPanel } from '../alerts/AlertsPanel';
import { widgetsForGroup } from './registry';
import type { DashboardContext, Focus, RangeKey } from './types';

export const ALERTS_TAB = 'alerts';

/**
 * Health profile body: the panel of the selected tab. Tabs themselves are
 * chosen in the side bar (`ProfileSidebar`); one panel per metric group
 * (from the backend catalog) plus the panel listing every alert.
 */
export function ProfileTabs({
  astronautId,
  overview,
  catalog,
  groups,
  alerts,
  tab,
  focus,
  onOpenAlert,
}: {
  astronautId: string;
  overview: Overview;
  catalog: MetricDefinition[];
  groups: MetricGroup[];
  alerts: Alert[];
  tab: string;
  focus: Focus | null;
  onOpenAlert: (alert: Alert) => void;
}) {
  const [range, setRange] = useState<RangeKey>('90d');
  const ctx: DashboardContext = { astronautId, overview, catalog, range, focus };

  // Bring the alert's target into view once its panel has rendered.
  useEffect(() => {
    if (!focus) return;
    const id = requestAnimationFrame(() =>
      document.getElementById(`widget-${focus.metric}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' }),
    );
    return () => cancelAnimationFrame(id);
  }, [focus, tab]);

  if (tab === ALERTS_TAB) {
    return (
      <>
        <Title order={3} mb="md">
          Alerty
        </Title>
        <AlertsPanel alerts={alerts} groups={groups} onOpenAlert={onOpenAlert} />
      </>
    );
  }
  const group = groups.find((g) => g.code === tab);
  return group ? <GroupPanel group={group} ctx={ctx} onRangeChange={setRange} /> : null;
}

function GroupPanel({
  group,
  ctx,
  onRangeChange,
}: {
  group: MetricGroup;
  ctx: DashboardContext;
  onRangeChange: (r: RangeKey) => void;
}) {
  const widgets = useMemo(() => widgetsForGroup(group.code, ctx.catalog), [group.code, ctx.catalog]);
  return (
    <>
      <Group justify="space-between" mb="md" gap="xs" align="flex-end">
        <div>
          <Title order={3}>{group.name}</Title>
          <Text size="sm" c="dimmed">
            {group.description}
          </Text>
        </div>
        {widgets.some((w) => w.usesRange) && (
          <SegmentedControl
            size="xs"
            value={ctx.range}
            onChange={(v) => onRangeChange(v as RangeKey)}
            data={[
              { value: '30d', label: '30 dni' },
              { value: '90d', label: '90 dni' },
              { value: 'all', label: 'Cała misja' },
            ]}
          />
        )}
      </Group>
      <Grid gutter="md">
        {widgets.map(({ id, span, Component }) => {
          const focused = ctx.focus?.metric === id;
          return (
            <Grid.Col key={id} span={{ base: 12, ...span }}>
              <div
                id={`widget-${id}`}
                style={{
                  height: '100%',
                  scrollMarginTop: 76,
                  borderRadius: 'var(--mantine-radius-md)',
                  outline: focused ? '2px solid var(--mantine-primary-color-filled)' : 'none',
                  outlineOffset: 2,
                  transition: 'outline-color 300ms',
                }}
              >
                <Component ctx={ctx} />
              </div>
            </Grid.Col>
          );
        })}
      </Grid>
    </>
  );
}
