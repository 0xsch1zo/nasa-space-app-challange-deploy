//! JSON contracts. Mapping is mechanical and metric-agnostic: new metrics,
//! groups and alert rules flow through these types without changes.

use std::collections::BTreeMap;

use application::ports::inbound::{AlertCounts, CrewMemberSummary, HealthOverview, TimeSeries};
use application::security::Principal;
use chrono::{DateTime, NaiveDate, Utc};
use domain::alerts::Alert;
use domain::crew::Astronaut;
use domain::metrics::{Bounds, ComponentDefinition, MetricDefinition, MetricGroup};
use domain::wellbeing::WellbeingNote;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: &'static str,
    pub station_id: Uuid,
    pub astronaut_id: Option<Uuid>,
}

impl From<&Principal> for UserDto {
    fn from(p: &Principal) -> Self {
        Self {
            id: p.user_id.0,
            username: p.username.clone(),
            display_name: p.display_name.clone(),
            role: p.role.as_str(),
            station_id: p.station_id.0,
            astronaut_id: p.astronaut_id.map(|a| a.0),
        }
    }
}

// ---------- metric catalog ----------

#[derive(Serialize)]
pub struct BoundsDto {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl From<Bounds> for BoundsDto {
    fn from(b: Bounds) -> Self {
        Self {
            min: b.min,
            max: b.max,
        }
    }
}

#[derive(Serialize)]
pub struct ReferenceDto {
    pub normal: BoundsDto,
    pub warning: BoundsDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDto {
    pub key: String,
    pub label: String,
    pub unit: String,
    pub body_region: Option<&'static str>,
    pub decimals: u8,
    pub reference: ReferenceDto,
    pub derived: bool,
    /// Maximum contribution to the metric's derived score.
    pub score_max: Option<f64>,
}

impl From<&ComponentDefinition> for ComponentDto {
    fn from(c: &ComponentDefinition) -> Self {
        Self {
            key: c.key.clone(),
            label: c.label.clone(),
            unit: c.unit.clone(),
            body_region: c.body_region.map(|r| r.as_str()),
            decimals: c.decimals,
            reference: ReferenceDto {
                normal: c.reference.normal.into(),
                warning: c.reference.warning.into(),
            },
            derived: c.derived,
            score_max: c.score.map(|s| s.max_points),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricDefinitionDto {
    pub code: String,
    pub name: String,
    pub description: String,
    pub chart: &'static str,
    pub group: String,
    pub scope: &'static str,
    pub sampling_interval_hours: u32,
    pub components: Vec<ComponentDto>,
}

impl From<&MetricDefinition> for MetricDefinitionDto {
    fn from(d: &MetricDefinition) -> Self {
        Self {
            code: d.code.to_string(),
            name: d.name.clone(),
            description: d.description.clone(),
            chart: d.chart.as_str(),
            group: d.group.clone(),
            scope: d.scope.as_str(),
            sampling_interval_hours: d.sampling_interval_hours,
            components: d.components.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricGroupDto {
    pub code: String,
    pub name: String,
    pub description: String,
    pub default: bool,
}

impl From<&MetricGroup> for MetricGroupDto {
    fn from(g: &MetricGroup) -> Self {
        Self {
            code: g.code.clone(),
            name: g.name.clone(),
            description: g.description.clone(),
            default: g.default,
        }
    }
}

// ---------- crew ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstronautDto {
    pub id: Uuid,
    pub full_name: String,
    pub agency: String,
    pub specialty: String,
    pub mission_start: NaiveDate,
    pub station_id: Uuid,
}

impl From<&Astronaut> for AstronautDto {
    fn from(a: &Astronaut) -> Self {
        Self {
            id: a.id.0,
            full_name: a.full_name.clone(),
            agency: a.agency.clone(),
            specialty: a.specialty.clone(),
            mission_start: a.mission_start,
            station_id: a.station_id.0,
        }
    }
}

#[derive(Serialize)]
pub struct AlertCountsDto {
    pub critical: usize,
    pub warning: usize,
    pub notice: usize,
}

impl From<AlertCounts> for AlertCountsDto {
    fn from(c: AlertCounts) -> Self {
        Self {
            critical: c.critical,
            warning: c.warning,
            notice: c.notice,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrewMemberDto {
    #[serde(flatten)]
    pub astronaut: AstronautDto,
    pub station_name: String,
    pub mission_day: i64,
    pub overall_status: &'static str,
    pub last_measurement_at: Option<DateTime<Utc>>,
    pub metric_statuses: BTreeMap<String, &'static str>,
    pub alerts: AlertCountsDto,
}

impl From<CrewMemberSummary> for CrewMemberDto {
    fn from(s: CrewMemberSummary) -> Self {
        Self {
            astronaut: (&s.astronaut).into(),
            station_name: s.station.name,
            mission_day: s.mission_day,
            overall_status: s.overall_status.as_str(),
            last_measurement_at: s.last_measurement_at,
            metric_statuses: s
                .metric_statuses
                .into_iter()
                .map(|(code, st)| (code.to_string(), st.as_str()))
                .collect(),
            alerts: s.alert_counts.into(),
        }
    }
}

// ---------- health ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricSnapshotDto {
    pub code: String,
    pub recorded_at: Option<DateTime<Utc>>,
    pub values: BTreeMap<String, f64>,
    pub status: &'static str,
    pub component_statuses: BTreeMap<String, &'static str>,
    pub stale: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyFindingDto {
    pub metric: String,
    pub component: String,
    pub value: f64,
    pub status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyRegionDto {
    pub region: &'static str,
    pub status: &'static str,
    pub findings: Vec<BodyFindingDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDto {
    pub astronaut: AstronautDto,
    pub overall_status: &'static str,
    pub metrics: Vec<MetricSnapshotDto>,
    pub body: Vec<BodyRegionDto>,
}

impl From<HealthOverview> for OverviewDto {
    fn from(o: HealthOverview) -> Self {
        Self {
            astronaut: (&o.astronaut).into(),
            overall_status: o.overall_status.as_str(),
            metrics: o
                .metrics
                .into_iter()
                .map(|m| MetricSnapshotDto {
                    code: m.metric.to_string(),
                    recorded_at: m.latest.as_ref().map(|r| r.recorded_at),
                    values: m.latest.map(|r| r.values).unwrap_or_default(),
                    status: m
                        .assessment
                        .as_ref()
                        .map(|a| a.status.as_str())
                        .unwrap_or("unknown"),
                    component_statuses: m
                        .assessment
                        .map(|a| {
                            a.components
                                .into_iter()
                                .map(|c| (c.key, c.status.as_str()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    stale: m.stale,
                })
                .collect(),
            body: o
                .body
                .into_iter()
                .map(|r| BodyRegionDto {
                    region: r.region.as_str(),
                    status: r.status.as_str(),
                    findings: r
                        .findings
                        .into_iter()
                        .map(|f| BodyFindingDto {
                            metric: f.metric.to_string(),
                            component: f.component,
                            value: f.value,
                            status: f.status.as_str(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct SeriesParams {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub bucket: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesPointDto {
    pub t: DateTime<Utc>,
    pub values: BTreeMap<String, f64>,
    pub status: &'static str,
    pub statuses: BTreeMap<String, &'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDto {
    pub metric: String,
    pub points: Vec<SeriesPointDto>,
}

impl From<TimeSeries> for SeriesDto {
    fn from(s: TimeSeries) -> Self {
        Self {
            metric: s.metric.to_string(),
            points: s
                .points
                .into_iter()
                .map(|p| SeriesPointDto {
                    t: p.reading.recorded_at,
                    values: p.reading.values,
                    status: p.assessment.status.as_str(),
                    statuses: p
                        .assessment
                        .components
                        .into_iter()
                        .map(|c| (c.key, c.status.as_str()))
                        .collect(),
                })
                .collect(),
        }
    }
}

// ---------- alerts ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertFindingDto {
    pub component: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub status: &'static str,
    pub normal: BoundsDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertDto {
    pub id: String,
    pub severity: &'static str,
    pub metric: String,
    pub group: String,
    pub title: String,
    pub message: String,
    pub recorded_at: Option<DateTime<Utc>>,
    pub findings: Vec<AlertFindingDto>,
}

impl From<Alert> for AlertDto {
    fn from(a: Alert) -> Self {
        Self {
            id: a.id,
            severity: a.severity.as_str(),
            metric: a.metric.to_string(),
            group: a.group,
            title: a.title,
            message: a.message,
            recorded_at: a.recorded_at,
            findings: a
                .findings
                .into_iter()
                .map(|f| AlertFindingDto {
                    component: f.component,
                    label: f.label,
                    value: f.value,
                    unit: f.unit,
                    status: f.status.as_str(),
                    normal: f.normal.into(),
                })
                .collect(),
        }
    }
}

// ---------- wellbeing ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WellbeingNoteDto {
    pub written_at: DateTime<Utc>,
    pub body: String,
}

impl From<WellbeingNote> for WellbeingNoteDto {
    fn from(n: WellbeingNote) -> Self {
        Self {
            written_at: n.written_at,
            body: n.body,
        }
    }
}
