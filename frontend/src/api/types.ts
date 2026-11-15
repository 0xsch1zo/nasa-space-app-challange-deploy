// Mirrors the backend JSON contracts (adapter-http/src/dto.rs).
// Nothing here is metric-specific: metrics are described at runtime by the catalog.

export type Role = 'astronaut' | 'supervisor';
export type HealthStatus = 'unknown' | 'normal' | 'warning' | 'critical';

export interface User {
  id: string;
  username: string;
  displayName: string;
  role: Role;
  stationId: string;
  astronautId: string | null;
}

export interface LoginResponse {
  token: string;
  expiresAt: string;
  user: User;
}

export interface Bounds {
  min: number | null;
  max: number | null;
}

export interface MetricComponent {
  key: string;
  label: string;
  unit: string;
  /** Body part the value is drawn on (body-map metrics), otherwise null. */
  bodyRegion: string | null;
  decimals: number;
  reference: { normal: Bounds; warning: Bounds };
  /** Computed by the backend from other components (e.g. sleep score). */
  derived: boolean;
  scoreMax: number | null;
}

export type ChartKind = 'line' | 'area' | 'bar' | 'table' | 'body';

export interface MetricDefinition {
  code: string;
  name: string;
  description: string;
  chart: ChartKind;
  /** Dashboard tab (MetricGroup.code). */
  group: string;
  scope: 'astronaut' | 'station';
  samplingIntervalHours: number;
  components: MetricComponent[];
}

export interface MetricGroup {
  code: string;
  name: string;
  description: string;
  default: boolean;
}

export type AlertSeverity = 'critical' | 'warning' | 'notice';

export interface AlertFinding {
  component: string;
  label: string;
  value: number;
  unit: string;
  status: HealthStatus;
  normal: Bounds;
}

export interface Alert {
  id: string;
  severity: AlertSeverity;
  metric: string;
  group: string;
  title: string;
  message: string;
  recordedAt: string | null;
  findings: AlertFinding[];
}

export interface AlertCounts {
  critical: number;
  warning: number;
  notice: number;
}

export interface WellbeingNote {
  writtenAt: string;
  body: string;
}

export interface Astronaut {
  id: string;
  fullName: string;
  agency: string;
  specialty: string;
  missionStart: string;
  stationId: string;
}

export interface CrewMember extends Astronaut {
  stationName: string;
  missionDay: number;
  overallStatus: HealthStatus;
  lastMeasurementAt: string | null;
  metricStatuses: Record<string, HealthStatus>;
  alerts: AlertCounts;
}

export interface MetricSnapshot {
  code: string;
  recordedAt: string | null;
  values: Record<string, number>;
  status: HealthStatus;
  componentStatuses: Record<string, HealthStatus>;
  stale: boolean;
}

export interface BodyFinding {
  metric: string;
  component: string;
  value: number;
  status: HealthStatus;
}

export interface BodyRegionSummary {
  region: string;
  status: HealthStatus;
  findings: BodyFinding[];
}

export interface Overview {
  astronaut: Astronaut;
  overallStatus: HealthStatus;
  metrics: MetricSnapshot[];
  body: BodyRegionSummary[];
}

export interface SeriesPoint {
  t: string;
  values: Record<string, number>;
  /** Status of the whole reading and of every component (from the backend). */
  status: HealthStatus;
  statuses: Record<string, HealthStatus>;
}

export interface Series {
  metric: string;
  points: SeriesPoint[];
}
