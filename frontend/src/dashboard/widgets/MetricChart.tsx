import { AreaChart, BarChart, LineChart } from '@mantine/charts';
import { Center, Text } from '@mantine/core';
import { useSeries, type SeriesParams } from '../../api/hooks';
import type { MetricDefinition } from '../../api/types';
import { QueryState } from '../../components/QueryState';
import { dayjs, fmtShort } from '../../lib/dates';
import type { RangeKey } from '../types';

const PALETTE = ['indigo.4', 'cyan.4', 'grape.4', 'orange.4', 'lime.4', 'pink.4'];

function seriesParams(metric: MetricDefinition, range: RangeKey): SeriesParams {
  const days = range === '30d' ? 30 : range === '90d' ? 90 : null;
  return {
    // Rounded to the day so the query key is stable between renders.
    from: days ? dayjs().startOf('day').subtract(days, 'day').toISOString() : undefined,
    // Downsample sub-daily metrics over long ranges; weekly/daily stay raw.
    bucket: range === 'all' && metric.samplingIntervalHours < 24 ? 'day' : undefined,
  };
}

/**
 * Generic time-series chart driven entirely by the metric definition:
 * one series per component, reference lines from the clinical ranges.
 */
export function MetricChart({
  astronautId,
  metric,
  range,
  height = 220,
  components,
  compact = false,
}: {
  astronautId: string;
  metric: MetricDefinition;
  range: RangeKey;
  height?: number;
  /** Plot only these component keys (default: all measured components). */
  components?: string[];
  /** Small chart without legend and X axis, e.g. inside a tooltip. */
  compact?: boolean;
}) {
  const series = useSeries(astronautId, metric.code, seriesParams(metric, range));
  const shown = metric.components.filter((c) =>
    components ? components.includes(c.key) : !c.derived,
  );

  const data =
    series.data?.points.map((p) => {
      const row: Record<string, string | number> = { t: fmtShort(p.t) };
      for (const c of shown) {
        const v = p.values[c.key];
        if (v !== undefined) row[c.key] = Number(v.toFixed(c.decimals));
      }
      return row;
    }) ?? [];

  // Reference lines: normal bounds per component; warning bounds as well when
  // all components share one reference (otherwise the chart gets cluttered).
  const lines = new Map<number, { y: number; label: string; color: string }>();
  const sharedRef =
    new Set(shown.map((c) => JSON.stringify(c.reference))).size === 1;
  for (const c of shown) {
    for (const y of [c.reference.normal.min, c.reference.normal.max]) {
      if (y !== null && !lines.has(y)) lines.set(y, { y, label: `${y}`, color: 'yellow.6' });
    }
    if (sharedRef) {
      for (const y of [c.reference.warning.min, c.reference.warning.max]) {
        if (y !== null && !lines.has(y)) lines.set(y, { y, label: `${y}`, color: 'red.6' });
      }
    }
  }

  // Y axis with "nice" ticks that always shows data and reference lines.
  const values = data
    .flatMap((r) => shown.map((c) => r[c.key]))
    .filter((v): v is number => typeof v === 'number');
  // Reference lines far from the data would flatten the curve (e.g. gravity
  // ~1 µg vs a 100 µg limit); only lines near the data widen the axis.
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const reach = 2 * Math.max(hi - lo, Math.abs(hi + lo) * 0.05, 1e-6);
  const near = [...lines.keys()].filter((y) => y >= lo - reach && y <= hi + reach);
  const { domain, ticks } = niceAxis([...values, ...near]);

  const decimals = Math.max(...shown.map((c) => c.decimals));
  const common = {
    h: height,
    data,
    dataKey: 't',
    series: shown.map((c, i) => ({
      name: c.key,
      label: c.label,
      color: PALETTE[i % PALETTE.length],
    })),
    referenceLines: [...lines.values()].filter((l) => near.includes(l.y)),
    withLegend: !compact && shown.length > 1,
    withXAxis: !compact,
    legendProps: {
      verticalAlign: 'bottom' as const,
      height: 40,
      // keep the order of components from the metric definition
      itemSorter: (item: { dataKey?: unknown }) =>
        shown.findIndex((c) => c.key === item.dataKey),
    },
    valueFormatter: (v: number) => `${v.toFixed(decimals)} ${shown[0].unit}`,
    yAxisProps: { domain, ticks, width: 44, tickFormatter: (v: number) => `${Number(v.toFixed(3))}` },
    xAxisProps: { minTickGap: 24 },
    gridAxis: 'y' as const,
  };

  return (
    <QueryState isLoading={series.isLoading} error={series.error} minHeight={height}>
      {data.length === 0 ? (
        <Center h={height}>
          <Text c="dimmed" size="sm">
            Brak pomiarów w wybranym okresie
          </Text>
        </Center>
      ) : metric.chart === 'bar' ? (
        <BarChart {...common} />
      ) : metric.chart === 'area' ? (
        <AreaChart
          {...common}
          curveType="monotone"
          withDots={false}
          fillOpacity={0.15}
          areaProps={{ baseValue: 'dataMin' }}
        />
      ) : (
        <LineChart {...common} curveType="monotone" withDots={data.length < 40} />
      )}
    </QueryState>
  );
}

/** Round axis bounds to a 1/2/2.5/5 × 10^k step and list the ticks explicitly. */
function niceAxis(values: number[]): { domain: [number, number]; ticks: number[] } {
  if (values.length === 0) return { domain: [0, 1], ticks: [0, 1] };
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const raw = (hi - lo || Math.abs(hi) || 1) / 4;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? 10 * mag;
  const min = Math.floor(lo / step) * step;
  const max = Math.ceil(hi / step) * step;
  const ticks: number[] = [];
  for (let t = min; t <= max + step / 2; t += step) ticks.push(Number(t.toFixed(6)));
  return { domain: [min, max], ticks };
}
