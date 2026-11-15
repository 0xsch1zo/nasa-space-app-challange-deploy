use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use super::definition::{HealthStatus, MetricCode, MetricDefinition, MetricScope};
use crate::crew::Subject;
use crate::error::DomainError;

/// A single point in a metric's history: component key → numeric value.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub recorded_at: DateTime<Utc>,
    pub values: BTreeMap<String, f64>,
}

/// A reading that passed domain validation and may be appended to the
/// `(astronaut, metric)` history aggregate. Only constructible through
/// [`MetricHistory::record`], which enforces the metric's invariants.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedReading {
    subject: Subject,
    metric: MetricCode,
    reading: Reading,
    source: String,
}

impl RecordedReading {
    pub fn subject(&self) -> Subject {
        self.subject
    }
    pub fn metric(&self) -> &MetricCode {
        &self.metric
    }
    pub fn reading(&self) -> &Reading {
        &self.reading
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// The historical aggregate of one health metric for one subject
/// (an astronaut, or a station for environment metrics).
///
/// History is append-only and potentially long, so the aggregate is never
/// loaded as a whole for writes: `record` validates a new reading against the
/// metric definition and yields a [`RecordedReading`] for the repository to
/// append. Reads go through dedicated read models (time series, latest value).
pub struct MetricHistory;

impl MetricHistory {
    pub fn record(
        definition: &MetricDefinition,
        subject: Subject,
        mut reading: Reading,
        source: impl Into<String>,
    ) -> Result<RecordedReading, DomainError> {
        let metric = definition.code.as_str();
        let scope_ok = matches!(
            (definition.scope, subject),
            (MetricScope::Astronaut, Subject::Astronaut(_))
                | (MetricScope::Station, Subject::Station(_))
        );
        if !scope_ok {
            return Err(DomainError::ScopeMismatch(metric.to_owned()));
        }
        for key in reading.values.keys() {
            match definition.component(key) {
                None => {
                    return Err(DomainError::UnknownComponent {
                        metric: metric.to_owned(),
                        component: key.clone(),
                    })
                }
                Some(c) if c.derived => {
                    return Err(DomainError::DerivedComponentProvided {
                        metric: metric.to_owned(),
                        component: key.clone(),
                    })
                }
                Some(_) => {}
            }
        }
        for c in definition.components.iter().filter(|c| !c.derived) {
            let value =
                *reading
                    .values
                    .get(&c.key)
                    .ok_or_else(|| DomainError::MissingComponent {
                        metric: metric.to_owned(),
                        component: c.key.clone(),
                    })?;
            if !value.is_finite() || !c.plausible.contains(value) {
                return Err(DomainError::ImplausibleValue {
                    metric: metric.to_owned(),
                    component: c.key.clone(),
                    value,
                    min: c.plausible.min.unwrap_or(f64::NEG_INFINITY),
                    max: c.plausible.max.unwrap_or(f64::INFINITY),
                });
            }
        }
        // Derived components (e.g. the overall sleep score) are the sum of the
        // per-component scores, computed here so every adapter gets them.
        let score: f64 = definition
            .components
            .iter()
            .filter_map(|c| Some(c.score?.points(*reading.values.get(&c.key)?)))
            .sum();
        for c in definition.components.iter().filter(|c| c.derived) {
            reading.values.insert(c.key.clone(), score.round());
        }
        Ok(RecordedReading {
            subject,
            metric: definition.code.clone(),
            reading,
            source: source.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentAssessment {
    pub key: String,
    pub value: f64,
    pub status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assessment {
    pub status: HealthStatus,
    pub components: Vec<ComponentAssessment>,
}

impl MetricDefinition {
    /// Classifies every component of a reading and the reading as a whole.
    /// The overall status is the worst of the components covered by the
    /// metric's alert policy (e.g. only the sleep score), so the headline
    /// status and the alerts always agree.
    pub fn assess(&self, reading: &Reading) -> Assessment {
        let components: Vec<_> = self
            .components
            .iter()
            .filter_map(|c| {
                reading
                    .values
                    .get(&c.key)
                    .map(|&value| ComponentAssessment {
                        key: c.key.clone(),
                        value,
                        status: c.reference.classify(value),
                    })
            })
            .collect();
        let status = components
            .iter()
            .filter(|c| self.alert_policy.covers(&c.key))
            .map(|c| c.status)
            .max()
            .unwrap_or(HealthStatus::Unknown);
        Assessment { status, components }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricCatalog;

    fn reading(pairs: &[(&str, f64)]) -> Reading {
        Reading {
            recorded_at: Utc::now(),
            values: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn rejects_missing_unknown_and_implausible_components() {
        let catalog = MetricCatalog::builtin();
        let bp = catalog.get_str("blood_pressure").unwrap();
        let id = Subject::Astronaut(crate::crew::AstronautId::new());

        assert!(MetricHistory::record(bp, id, reading(&[("systolic", 120.0)]), "t").is_err());
        assert!(MetricHistory::record(
            bp,
            id,
            reading(&[("systolic", 120.0), ("diastolic", 80.0), ("pulse", 60.0)]),
            "t"
        )
        .is_err());
        assert!(MetricHistory::record(
            bp,
            id,
            reading(&[("systolic", 900.0), ("diastolic", 80.0)]),
            "t"
        )
        .is_err());
        assert!(MetricHistory::record(
            bp,
            id,
            reading(&[("systolic", 120.0), ("diastolic", 80.0)]),
            "t"
        )
        .is_ok());
    }

    #[test]
    fn worst_component_determines_status() {
        let catalog = MetricCatalog::builtin();
        let bp = catalog.get_str("blood_pressure").unwrap();
        let a = bp.assess(&reading(&[("systolic", 118.0), ("diastolic", 95.0)]));
        assert_eq!(a.components[0].status, HealthStatus::Normal);
        assert!(a.status > HealthStatus::Normal);
    }

    #[test]
    fn derived_score_is_computed_and_cannot_be_sent() {
        let catalog = MetricCatalog::builtin();
        let sleep = catalog.get_str("sleep").unwrap();
        let id = Subject::Astronaut(crate::crew::AstronautId::new());
        let good = reading(&[
            ("total_sleep", 8.0),
            ("efficiency", 95.0),
            ("deep_sleep", 20.0),
            ("rem_sleep", 23.0),
            ("latency", 5.0),
            ("awakenings", 0.0),
        ]);
        let r = MetricHistory::record(sleep, id, good.clone(), "t").unwrap();
        assert_eq!(r.reading().values["score"], 100.0);

        let mut forged = good;
        forged.values.insert("score".into(), 100.0);
        assert!(MetricHistory::record(sleep, id, forged, "t").is_err());
    }

    #[test]
    fn scope_must_match_subject() {
        let catalog = MetricCatalog::builtin();
        let radiation = catalog.get_str("radiation").unwrap();
        let astronaut = Subject::Astronaut(crate::crew::AstronautId::new());
        let r = reading(&[("dose_rate", 0.4)]);
        assert!(MetricHistory::record(radiation, astronaut, r.clone(), "t").is_err());
        let station = Subject::Station(crate::crew::StationId::new());
        assert!(MetricHistory::record(radiation, station, r, "t").is_ok());
    }

    #[test]
    fn metric_code_is_restricted() {
        assert!(MetricCode::new("heart_rate").is_ok());
        assert!(MetricCode::new("Heart-Rate").is_err());
        assert!(MetricCode::new("x; drop table").is_err());
    }
}
