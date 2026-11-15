import type { ComponentType } from 'react';
import type { MetricDefinition, MetricSnapshot, Overview } from '../api/types';

export type RangeKey = '30d' | '90d' | 'all';

/** Where an alert link points to: a metric widget, optionally one component. */
export interface Focus {
  metric: string;
  component?: string;
  /** Changes on every navigation so repeated clicks re-scroll. */
  nonce: number;
}

/** Everything a widget may need. Widgets fetch anything else themselves. */
export interface DashboardContext {
  astronautId: string;
  overview: Overview;
  catalog: MetricDefinition[];
  range: RangeKey;
  focus: Focus | null;
}

export interface GridSpan {
  base?: number;
  md?: number;
  lg?: number;
}

/** A self-contained dashboard module. */
export interface WidgetModule {
  /** Metric code for metric widgets; the DOM anchor alert links scroll to. */
  id: string;
  span: GridSpan;
  /** Whether the tab's time-range selector affects this widget. */
  usesRange?: boolean;
  Component: ComponentType<{ ctx: DashboardContext }>;
}

export interface MetricWidgetProps {
  ctx: DashboardContext;
  metric: MetricDefinition;
  snapshot: MetricSnapshot | undefined;
}

export type MetricWidgetComponent = ComponentType<MetricWidgetProps>;
