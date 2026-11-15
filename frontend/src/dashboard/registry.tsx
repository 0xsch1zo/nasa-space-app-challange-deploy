import type { ChartKind, MetricDefinition } from '../api/types';
import type { GridSpan, MetricWidgetComponent, WidgetModule } from './types';
import { BloodPressureWidget } from './widgets/BloodPressureWidget';
import { bodyMetricWidget, BODY_PALETTES } from './widgets/BodyMetricWidget';
import { LabPanelWidget } from './widgets/LabPanelWidget';
import { mentalNoteWidget } from './widgets/MentalNoteWidget';
import { MetricChartWidget } from './widgets/MetricChartWidget';
import { SamplingWidget } from './widgets/SamplingWidget';
import { ScoreWidget } from './widgets/ScoreWidget';

/**
 * Widget registry — the only place the dashboard learns what to render.
 *
 * Tabs come from the backend (metric groups); every metric of the catalog is
 * placed in its group's tab automatically:
 *
 * 1. a custom widget from `metricWidgetOverrides`, if registered;
 * 2. otherwise `ScoreWidget` for metrics with a derived score;
 * 3. otherwise the default widget for its chart kind (`widgetsByChart`).
 *
 * A NEW METRIC THEREFORE NEEDS NO FRONTEND CHANGE. A NEW TAB needs only a
 * backend group; non-metric modules for a tab go to `groupWidgets`.
 */

/** Modules that are not tied to one metric, shown first in their tab. */
export const groupWidgets: Record<string, WidgetModule[]> = {
  wellbeing: [mentalNoteWidget],
};

export const metricWidgetOverrides: Record<string, MetricWidgetComponent> = {
  blood_pressure: BloodPressureWidget,
  surface_microbes: SamplingWidget,
  muscle_atrophy: bodyMetricWidget(BODY_PALETTES.muscle),
  // Same body view as muscles, different colours to tell them apart.
  bone_density: bodyMetricWidget(BODY_PALETTES.bone),
};

const widgetsByChart: Record<ChartKind, MetricWidgetComponent> = {
  line: MetricChartWidget,
  area: MetricChartWidget,
  bar: MetricChartWidget,
  table: LabPanelWidget,
  body: bodyMetricWidget(BODY_PALETTES.muscle),
};

/** Wide reports (scores, lab tables, many-series metrics) take the full width. */
const spanFor = (metric: MetricDefinition, hasScore: boolean): GridSpan =>
  hasScore || metric.chart === 'table' || metric.components.length >= 4
    ? { base: 12 }
    : { base: 12, lg: 6 };

export function metricWidget(metric: MetricDefinition): WidgetModule {
  const hasScore = metric.components.some((c) => c.derived);
  const Widget =
    metricWidgetOverrides[metric.code] ?? (hasScore ? ScoreWidget : widgetsByChart[metric.chart]);
  return {
    id: metric.code,
    span: spanFor(metric, hasScore),
    usesRange: metric.chart !== 'table',
    Component: ({ ctx }) => (
      <Widget
        ctx={ctx}
        metric={metric}
        snapshot={ctx.overview.metrics.find((m) => m.code === metric.code)}
      />
    ),
  };
}

export function widgetsForGroup(group: string, catalog: MetricDefinition[]): WidgetModule[] {
  return [
    ...(groupWidgets[group] ?? []),
    ...catalog.filter((m) => m.group === group).map(metricWidget),
  ];
}
