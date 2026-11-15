//! Alerts: which parameters need attention, and how urgently.
//!
//! Alerts are derived from metric definitions and the latest readings by a
//! set of [`AlertRule`]s. New metrics get alerts for free (their reference
//! ranges drive [`OutOfRangeRule`]); new kinds of alerts are a new rule
//! registered in [`AlertEngine::standard`].

use chrono::{DateTime, Duration, Utc};

use crate::metrics::{Bounds, HealthStatus, MetricCatalog, MetricCode, MetricDefinition, Reading};

/// Ordered from least to most urgent so that sorting descending puts the
/// most critical alerts first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertSeverity {
    /// Worth a look, no health risk (e.g. an overdue measurement).
    Notice,
    /// A value outside the normal range.
    Warning,
    /// A value outside the warning range.
    Critical,
}

impl AlertSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertSeverity::Notice => "notice",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        }
    }
}

/// One component value behind an alert.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertFinding {
    pub component: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub status: HealthStatus,
    pub normal: Bounds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    /// Stable id, `<metric>:<rule>`.
    pub id: String,
    pub severity: AlertSeverity,
    pub metric: MetricCode,
    /// Dashboard tab holding the full report of this metric.
    pub group: String,
    pub title: String,
    pub message: String,
    pub recorded_at: Option<DateTime<Utc>>,
    /// Worst finding first.
    pub findings: Vec<AlertFinding>,
}

pub trait AlertRule: Send + Sync {
    fn evaluate(
        &self,
        metric: &MetricDefinition,
        latest: Option<&Reading>,
        now: DateTime<Utc>,
    ) -> Option<Alert>;
}

/// Latest reading outside the reference range. One alert per metric (the
/// worst component is named, the rest summarised) to avoid flooding the view.
pub struct OutOfRangeRule;

impl AlertRule for OutOfRangeRule {
    fn evaluate(
        &self,
        def: &MetricDefinition,
        latest: Option<&Reading>,
        _: DateTime<Utc>,
    ) -> Option<Alert> {
        let reading = latest?;
        let mut findings: Vec<AlertFinding> = def
            .assess(reading)
            .components
            .into_iter()
            .filter(|c| c.status > HealthStatus::Normal && def.alert_policy.covers(&c.key))
            .filter_map(|c| {
                let comp = def.component(&c.key)?;
                Some(AlertFinding {
                    component: c.key,
                    label: comp.label.clone(),
                    value: round(c.value, comp.decimals),
                    unit: comp.unit.clone(),
                    status: c.status,
                    normal: comp.reference.normal,
                })
            })
            .collect();
        findings.sort_by_key(|f| std::cmp::Reverse(f.status));
        let worst = findings.first()?;
        let severity = if worst.status == HealthStatus::Critical {
            AlertSeverity::Critical
        } else {
            AlertSeverity::Warning
        };
        let mut message = format!(
            "{}: {} {} (norma {})",
            worst.label,
            worst.value,
            worst.unit,
            worst.normal.describe(&worst.unit)
        );
        if findings.len() > 1 {
            message.push_str(&format!(" · {}", others(findings.len() - 1)));
        }
        Some(Alert {
            id: format!("{}:out_of_range", def.code),
            severity,
            metric: def.code.clone(),
            group: def.group.clone(),
            title: def.name.clone(),
            message,
            recorded_at: Some(reading.recorded_at),
            findings,
        })
    }
}

/// No fresh data: the last reading is older than `factor` sampling intervals.
pub struct OverdueMeasurementRule {
    pub factor: i32,
}

impl AlertRule for OverdueMeasurementRule {
    fn evaluate(
        &self,
        def: &MetricDefinition,
        latest: Option<&Reading>,
        now: DateTime<Utc>,
    ) -> Option<Alert> {
        let reading = latest?;
        let interval = Duration::hours(i64::from(def.sampling_interval_hours));
        let age = now - reading.recorded_at;
        if age <= interval * self.factor {
            return None;
        }
        Some(Alert {
            id: format!("{}:overdue", def.code),
            severity: AlertSeverity::Notice,
            metric: def.code.clone(),
            group: def.group.clone(),
            title: def.name.clone(),
            message: format!(
                "Zaległy pomiar: ostatni {} temu, oczekiwany co {}",
                human_duration(age),
                human_duration(interval)
            ),
            recorded_at: Some(reading.recorded_at),
            findings: Vec::new(),
        })
    }
}

pub struct AlertEngine {
    rules: Vec<Box<dyn AlertRule>>,
}

impl AlertEngine {
    /// The rules applied in production.
    pub fn standard() -> Self {
        Self::new(vec![
            Box::new(OutOfRangeRule),
            Box::new(OverdueMeasurementRule { factor: 2 }),
        ])
    }

    pub fn new(rules: Vec<Box<dyn AlertRule>>) -> Self {
        Self { rules }
    }

    /// Alerts for one profile, most urgent first (ties: catalog order).
    pub fn evaluate(
        &self,
        catalog: &MetricCatalog,
        latest: &[(MetricCode, Reading)],
        now: DateTime<Utc>,
    ) -> Vec<Alert> {
        let mut alerts: Vec<(usize, Alert)> = Vec::new();
        for (idx, def) in catalog.all().iter().enumerate() {
            let reading = latest.iter().find(|(c, _)| c == &def.code).map(|(_, r)| r);
            for rule in &self.rules {
                if let Some(a) = rule.evaluate(def, reading, now) {
                    alerts.push((idx, a));
                }
            }
        }
        alerts.sort_by(|(ia, a), (ib, b)| b.severity.cmp(&a.severity).then(ia.cmp(ib)));
        alerts.into_iter().map(|(_, a)| a).collect()
    }
}

fn round(v: f64, decimals: u8) -> f64 {
    let f = 10f64.powi(i32::from(decimals));
    (v * f).round() / f
}

/// "1 inny wynik", "3 inne wyniki", "5 innych wyników" (Polish plural rules).
fn others(n: usize) -> String {
    let (adj, noun) = if n == 1 {
        ("inny", "wynik")
    } else if (2..=4).contains(&(n % 10)) && !(12..=14).contains(&(n % 100)) {
        ("inne", "wyniki")
    } else {
        ("innych", "wyników")
    };
    format!("oraz {n} {adj} {noun} poza normą")
}

fn human_duration(d: Duration) -> String {
    let hours = d.num_hours();
    if hours < 48 {
        format!("{hours} h")
    } else {
        format!("{} dni", d.num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(at: DateTime<Utc>, pairs: &[(&str, f64)]) -> Reading {
        Reading {
            recorded_at: at,
            values: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    fn code(c: &str) -> MetricCode {
        MetricCode::new(c).unwrap()
    }

    #[test]
    fn alerts_are_sorted_by_severity_and_summarised_per_metric() {
        let catalog = MetricCatalog::builtin();
        let now = Utc::now();
        let latest = vec![
            (
                code("blood_pressure"),
                reading(now, &[("systolic", 135.0), ("diastolic", 80.0)]),
            ),
            (code("body_temperature"), reading(now, &[("core", 38.6)])),
            (
                code("heart_rate"),
                reading(now - Duration::days(5), &[("resting", 60.0)]),
            ),
        ];
        let alerts = AlertEngine::standard().evaluate(&catalog, &latest, now);
        let kinds: Vec<_> = alerts
            .iter()
            .map(|a| (a.metric.as_str(), a.severity))
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("body_temperature", AlertSeverity::Critical),
                ("blood_pressure", AlertSeverity::Warning),
                ("heart_rate", AlertSeverity::Notice),
            ]
        );
        assert_eq!(alerts[1].group, "physiology");
    }

    #[test]
    fn alert_policy_limits_noisy_components() {
        let catalog = MetricCatalog::builtin();
        let sleep = catalog.get_str("sleep").unwrap();
        let now = Utc::now();
        // Short latency issue alone, good overall score: no alert.
        let r = reading(
            now,
            &[
                ("total_sleep", 7.8),
                ("efficiency", 93.0),
                ("deep_sleep", 19.0),
                ("rem_sleep", 22.0),
                ("latency", 45.0),
                ("awakenings", 1.0),
                ("score", 92.0),
            ],
        );
        assert!(OutOfRangeRule.evaluate(sleep, Some(&r), now).is_none());
    }

    #[test]
    fn polish_plurals() {
        assert_eq!(others(1), "oraz 1 inny wynik poza normą");
        assert_eq!(others(3), "oraz 3 inne wyniki poza normą");
        assert_eq!(others(5), "oraz 5 innych wyników poza normą");
        assert_eq!(others(12), "oraz 12 innych wyników poza normą");
    }
}
