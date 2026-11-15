import { useRef, useState, type MouseEvent } from 'react';
import { Badge, Grid, Group, Paper, Stack, Text, UnstyledButton } from '@mantine/core';
import type { HealthStatus, MetricComponent } from '../../api/types';
import { StatusBadge } from '../../components/StatusBadge';
import { formatBounds, formatValue, statusLabel, statusRank } from '../../lib/status';
import type { MetricWidgetComponent, MetricWidgetProps } from '../types';
import { WidgetCard } from '../WidgetCard';
import { BodyFigure, type PartStyle } from './BodyFigure';
import { MetricChart } from './MetricChart';
import { SnapshotBadge } from './MetricChartWidget';

/** Status → Mantine colour. Different palettes keep muscles and bones distinct. */
export type BodyPalette = Record<HealthStatus, string>;

export const BODY_PALETTES = {
  muscle: { normal: 'teal', warning: 'orange', critical: 'red', unknown: 'gray' },
  bone: { normal: 'cyan', warning: 'grape', critical: 'pink', unknown: 'gray' },
} satisfies Record<string, BodyPalette>;

const cssColor = (palette: BodyPalette, s: HealthStatus) => `var(--mantine-color-${palette[s]}-6)`;

/**
 * Body-map widget for metrics whose components map to body parts
 * (muscle atrophy, bone loss). Each part is coloured by its own status;
 * hovering it shows the part's trend chart, clicking pins it in the side panel.
 */
export function bodyMetricWidget(palette: BodyPalette): MetricWidgetComponent {
  return function BodyMetricWidget(props: MetricWidgetProps) {
    return <BodyMetric {...props} palette={palette} />;
  };
}

function BodyMetric({ ctx, metric, snapshot, palette }: MetricWidgetProps & { palette: BodyPalette }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const statusOf = (c: MetricComponent): HealthStatus => snapshot?.componentStatuses[c.key] ?? 'unknown';
  const parts = metric.components.filter((c) => c.bodyRegion);
  const worst = [...parts].sort((a, b) => statusRank[statusOf(b)] - statusRank[statusOf(a)])[0];
  const focused = ctx.focus?.metric === metric.code ? ctx.focus.component : undefined;
  const [selectedKey, setSelectedKey] = useState<string | undefined>(focused ?? worst?.key);
  const [hover, setHover] = useState<{ key: string; x: number; y: number } | null>(null);
  // Alert links re-target the selection.
  const [lastFocus, setLastFocus] = useState(ctx.focus?.nonce);
  if (ctx.focus?.nonce !== lastFocus) {
    setLastFocus(ctx.focus?.nonce);
    if (focused) setSelectedKey(focused);
  }

  const byRegion = new Map(parts.map((c) => [c.bodyRegion!, c]));
  const styles: Record<string, PartStyle> = Object.fromEntries(
    parts.map((c) => [c.bodyRegion!, { fill: cssColor(palette, statusOf(c)), label: c.label }]),
  );
  const selected = parts.find((c) => c.key === selectedKey);
  const hovered = hover ? parts.find((c) => c.key === hover.key) : undefined;

  function onHover(region: string | null, e?: MouseEvent<SVGElement>) {
    const c = region ? byRegion.get(region) : undefined;
    const box = containerRef.current?.getBoundingClientRect();
    if (!c || !e || !box) return setHover(null);
    setHover({ key: c.key, x: e.clientX - box.left, y: e.clientY - box.top });
  }

  return (
    <WidgetCard title={metric.name} subtitle={metric.description} right={<SnapshotBadge snapshot={snapshot} />}>
      <Grid>
        <Grid.Col span={{ base: 12, sm: 6 }}>
          <div ref={containerRef} style={{ position: 'relative' }}>
            <BodyFigure
              parts={styles}
              selected={selected?.bodyRegion ?? null}
              onSelect={(region) => setSelectedKey(byRegion.get(region)?.key)}
              onHover={onHover}
            />
            {hovered && hover && (
              <Paper
                withBorder
                shadow="md"
                p="xs"
                w={250}
                style={{
                  position: 'absolute',
                  left: Math.min(hover.x + 12, (containerRef.current?.clientWidth ?? 300) - 250),
                  top: hover.y + 12,
                  pointerEvents: 'none',
                  zIndex: 10,
                }}
              >
                <Group justify="space-between" wrap="nowrap" mb={4}>
                  <Text size="sm" fw={700}>
                    {hovered.label}
                  </Text>
                  <Text size="sm" fw={700} c={`${palette[statusOf(hovered)]}.5`}>
                    {formatValue(snapshot?.values[hovered.key], hovered)}
                  </Text>
                </Group>
                <MetricChart
                  astronautId={ctx.astronautId}
                  metric={metric}
                  range={ctx.range}
                  components={[hovered.key]}
                  compact
                  height={110}
                />
              </Paper>
            )}
          </div>
          <Group gap={6} justify="center" mt={4}>
            {(['normal', 'warning', 'critical'] as HealthStatus[]).map((s) => (
              <Badge key={s} size="xs" variant="dot" color={palette[s]}>
                {statusLabel[s]}
              </Badge>
            ))}
          </Group>
        </Grid.Col>

        <Grid.Col span={{ base: 12, sm: 6 }}>
          <Stack gap={4}>
            {parts.map((c) => (
              <UnstyledButton
                key={c.key}
                onClick={() => setSelectedKey(c.key)}
                px="xs"
                py={4}
                style={{
                  borderRadius: 'var(--mantine-radius-sm)',
                  background: c.key === selectedKey ? 'var(--mantine-color-default-hover)' : undefined,
                }}
              >
                <Group justify="space-between" wrap="nowrap">
                  <Group gap={8} wrap="nowrap">
                    <span
                      style={{
                        width: 10,
                        height: 10,
                        borderRadius: 3,
                        background: cssColor(palette, statusOf(c)),
                        flexShrink: 0,
                      }}
                    />
                    <Text size="sm">{c.label}</Text>
                  </Group>
                  <Text size="sm" fw={700} c={`${palette[statusOf(c)]}.5`}>
                    {formatValue(snapshot?.values[c.key], c)}
                  </Text>
                </Group>
              </UnstyledButton>
            ))}
          </Stack>
          {selected && (
            <Paper withBorder p="xs" mt="sm">
              <Group justify="space-between" mb={4} wrap="nowrap">
                <Text size="sm" fw={700}>
                  {selected.label}
                </Text>
                <StatusBadge status={statusOf(selected)} size="sm" />
              </Group>
              <Text size="xs" c="dimmed" mb={4}>
                Norma {formatBounds(selected.reference.normal, selected.unit)}
              </Text>
              <MetricChart
                astronautId={ctx.astronautId}
                metric={metric}
                range={ctx.range}
                components={[selected.key]}
                height={140}
              />
            </Paper>
          )}
        </Grid.Col>
      </Grid>
    </WidgetCard>
  );
}
