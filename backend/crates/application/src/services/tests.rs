//! Use-case tests with in-memory adapters: the hexagon runs without HTTP or a DB.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use domain::alerts::{AlertEngine, AlertSeverity};
use domain::crew::{Astronaut, AstronautId, Role, Station, StationId, Subject, UserId};
use domain::metrics::{MetricCatalog, MetricCode, MetricScope, Reading, RecordedReading};
use domain::wellbeing::WellbeingNote;

use super::*;
use crate::error::AppError;
use crate::ports::inbound::*;
use crate::ports::outbound::*;
use crate::security::Principal;

#[derive(Default)]
struct InMemory {
    stations: Vec<Station>,
    astronauts: Vec<Astronaut>,
    readings: Mutex<Vec<RecordedReading>>,
}

#[async_trait]
impl CrewRepository for InMemory {
    async fn find_astronaut(&self, id: AstronautId) -> InfraResult<Option<Astronaut>> {
        Ok(self.astronauts.iter().find(|a| a.id == id).cloned())
    }
    async fn list_astronauts(&self, station: StationId) -> InfraResult<Vec<Astronaut>> {
        Ok(self
            .astronauts
            .iter()
            .filter(|a| a.station_id == station)
            .cloned()
            .collect())
    }
    async fn find_station(&self, id: StationId) -> InfraResult<Option<Station>> {
        Ok(self.stations.iter().find(|s| s.id == id).cloned())
    }
}

#[async_trait]
impl MeasurementRepository for InMemory {
    async fn ensure_storage(&self, _: &[(MetricCode, MetricScope)]) -> InfraResult<()> {
        Ok(())
    }
    async fn append(&self, r: &[RecordedReading]) -> InfraResult<u64> {
        self.readings.lock().unwrap().extend_from_slice(r);
        Ok(r.len() as u64)
    }
    async fn latest(&self, subject: Subject) -> InfraResult<Vec<(MetricCode, Reading)>> {
        let mut out: BTreeMap<MetricCode, Reading> = BTreeMap::new();
        for r in self
            .readings
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.subject() == subject)
        {
            let e = out
                .entry(r.metric().clone())
                .or_insert_with(|| r.reading().clone());
            if r.reading().recorded_at > e.recorded_at {
                *e = r.reading().clone();
            }
        }
        Ok(out.into_iter().collect())
    }
    async fn latest_for_station(
        &self,
        station: StationId,
    ) -> InfraResult<Vec<(AstronautId, MetricCode, Reading)>> {
        let mut out = Vec::new();
        for a in self.astronauts.iter().filter(|a| a.station_id == station) {
            for (code, r) in self.latest(Subject::Astronaut(a.id)).await? {
                out.push((a.id, code, r));
            }
        }
        Ok(out)
    }
    async fn series(
        &self,
        subject: Subject,
        metric: &MetricCode,
        _: TimeRange,
        _: Option<Bucket>,
    ) -> InfraResult<Vec<Reading>> {
        Ok(self
            .readings
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.subject() == subject && r.metric() == metric)
            .map(|r| r.reading().clone())
            .collect())
    }
}

#[async_trait]
impl WellbeingRepository for InMemory {
    async fn latest_note(&self, astronaut: AstronautId) -> InfraResult<Option<WellbeingNote>> {
        Ok(Some(WellbeingNote {
            astronaut_id: astronaut,
            written_at: Utc::now(),
            body: "Spokojny tydzień.".into(),
        }))
    }
}

struct Fixture {
    repo: Arc<InMemory>,
    iss: StationId,
    alice: AstronautId,
    bob: AstronautId,
    gateway_astro: AstronautId,
}

fn fixture() -> Fixture {
    let iss = StationId::new();
    let gateway = StationId::new();
    let mk = |station, name: &str| Astronaut {
        id: AstronautId::new(),
        station_id: station,
        full_name: name.into(),
        agency: "TEST".into(),
        specialty: "".into(),
        mission_start: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
    };
    let (a, b, g) = (mk(iss, "Alice"), mk(iss, "Bob"), mk(gateway, "Gina"));
    let ids = (a.id, b.id, g.id);
    let repo = Arc::new(InMemory {
        stations: vec![
            Station {
                id: iss,
                name: "ISS".into(),
            },
            Station {
                id: gateway,
                name: "Gateway".into(),
            },
        ],
        astronauts: vec![a, b, g],
        readings: Mutex::default(),
    });
    Fixture {
        repo,
        iss,
        alice: ids.0,
        bob: ids.1,
        gateway_astro: ids.2,
    }
}

fn astronaut(station: StationId, id: AstronautId) -> Principal {
    Principal {
        user_id: UserId::new(),
        username: "a".into(),
        display_name: "A".into(),
        role: Role::Astronaut,
        station_id: station,
        astronaut_id: Some(id),
    }
}

fn supervisor(station: StationId) -> Principal {
    Principal {
        user_id: UserId::new(),
        username: "s".into(),
        display_name: "S".into(),
        role: Role::Supervisor,
        station_id: station,
        astronaut_id: None,
    }
}

fn services(f: &Fixture) -> (HealthService, CrewService, IngestionService) {
    let catalog = Arc::new(MetricCatalog::builtin());
    let engine = Arc::new(AlertEngine::standard());
    (
        HealthService::new(f.repo.clone(), f.repo.clone(), catalog.clone()),
        CrewService::new(f.repo.clone(), f.repo.clone(), catalog.clone(), engine),
        IngestionService::new(f.repo.clone(), f.repo.clone(), catalog),
    )
}

fn alert_service(f: &Fixture) -> AlertService {
    AlertService::new(
        f.repo.clone(),
        f.repo.clone(),
        Arc::new(MetricCatalog::builtin()),
        Arc::new(AlertEngine::standard()),
    )
}

fn measurement(subject: Subject, metric: &str, values: &[(&str, f64)]) -> DeviceMeasurement {
    DeviceMeasurement {
        subject,
        metric: metric.into(),
        recorded_at: Utc::now(),
        values: values.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        source: "test".into(),
    }
}

#[tokio::test]
async fn astronaut_sees_only_own_profile() {
    let f = fixture();
    let (health, crew, _) = services(&f);
    let me = astronaut(f.iss, f.alice);

    assert!(health.overview(&me, f.alice).await.is_ok());
    assert!(matches!(
        health.overview(&me, f.bob).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        crew.list_crew(&me).await,
        Err(AppError::Forbidden)
    ));
    let q = SeriesQuery {
        astronaut: f.bob,
        metric: "blood_pressure".into(),
        range: TimeRange::default(),
        bucket: None,
    };
    assert!(matches!(
        health.series(&me, q).await,
        Err(AppError::Forbidden)
    ));
}

#[tokio::test]
async fn supervisor_sees_whole_station_but_not_other_stations() {
    let f = fixture();
    let (health, crew, _) = services(&f);
    let sup = supervisor(f.iss);

    let list = crew.list_crew(&sup).await.unwrap();
    assert_eq!(list.len(), 2);
    assert!(health.overview(&sup, f.bob).await.is_ok());
    assert!(matches!(
        health.overview(&sup, f.gateway_astro).await,
        Err(AppError::Forbidden)
    ));
}

#[tokio::test]
async fn ingestion_validates_against_catalog() {
    let f = fixture();
    let (health, _, ingest) = services(&f);
    let m = |metric: &str, values: &[(&str, f64)]| {
        measurement(Subject::Astronaut(f.alice), metric, values)
    };
    let report = ingest
        .ingest(vec![
            m(
                "blood_pressure",
                &[("systolic", 121.0), ("diastolic", 79.0)],
            ),
            m("blood_pressure", &[("systolic", 121.0)]),
            m("no_such_metric", &[("x", 1.0)]),
        ])
        .await
        .unwrap();
    assert_eq!(report.accepted, 1);
    assert_eq!(report.rejected.len(), 2);

    let overview = health
        .overview(&astronaut(f.iss, f.alice), f.alice)
        .await
        .unwrap();
    let bp = overview
        .metrics
        .iter()
        .find(|m| m.metric.as_str() == "blood_pressure")
        .unwrap();
    assert!(bp.latest.is_some());
}

#[tokio::test]
async fn alerts_include_station_environment_and_respect_access() {
    let f = fixture();
    let (_, crew, ingest) = services(&f);
    let alerts = alert_service(&f);
    let report = ingest
        .ingest(vec![
            measurement(
                Subject::Astronaut(f.alice),
                "body_temperature",
                &[("core", 38.6)],
            ),
            measurement(Subject::Station(f.iss), "radiation", &[("dose_rate", 1.4)]),
            // environment metric sent for an astronaut: rejected by the domain
            measurement(
                Subject::Astronaut(f.alice),
                "radiation",
                &[("dose_rate", 1.4)],
            ),
        ])
        .await
        .unwrap();
    assert_eq!(report.accepted, 2);
    assert_eq!(report.rejected.len(), 1);

    let mine = alerts
        .alerts(&astronaut(f.iss, f.alice), f.alice)
        .await
        .unwrap();
    let kinds: Vec<_> = mine
        .iter()
        .map(|a| (a.metric.as_str(), a.severity))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("body_temperature", AlertSeverity::Critical),
            ("radiation", AlertSeverity::Warning)
        ]
    );

    // Bob is on the same station: he gets the radiation alert only.
    let bob = alerts.alerts(&supervisor(f.iss), f.bob).await.unwrap();
    assert_eq!(bob.len(), 1);
    // Other astronauts and other stations stay inaccessible.
    assert!(matches!(
        alerts.alerts(&astronaut(f.iss, f.alice), f.bob).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        alerts.alerts(&supervisor(f.iss), f.gateway_astro).await,
        Err(AppError::Forbidden)
    ));

    let list = crew.list_crew(&supervisor(f.iss)).await.unwrap();
    let alice = list.iter().find(|c| c.astronaut.id == f.alice).unwrap();
    assert_eq!(
        (alice.alert_counts.critical, alice.alert_counts.warning),
        (1, 1)
    );
}

#[tokio::test]
async fn wellbeing_note_follows_access_policy() {
    let f = fixture();
    let svc = WellbeingService::new(f.repo.clone(), f.repo.clone());
    assert!(svc
        .latest_note(&astronaut(f.iss, f.alice), f.alice)
        .await
        .unwrap()
        .is_some());
    assert!(matches!(
        svc.latest_note(&astronaut(f.iss, f.alice), f.bob).await,
        Err(AppError::Forbidden)
    ));
}

#[tokio::test]
async fn stale_session_is_unauthenticated_not_not_found() {
    // Token issued before the data was reset: its station / astronaut no longer exist.
    let f = fixture();
    let (health, crew, _) = services(&f);
    assert!(matches!(
        crew.list_crew(&supervisor(StationId::new())).await,
        Err(AppError::Unauthenticated)
    ));
    let gone = AstronautId::new();
    assert!(matches!(
        health.overview(&astronaut(f.iss, gone), gone).await,
        Err(AppError::Unauthenticated)
    ));
    // A supervisor asking for an unknown astronaut is still a plain 404.
    assert!(matches!(
        health
            .overview(&supervisor(f.iss), AstronautId::new())
            .await,
        Err(AppError::NotFound(_))
    ));
}
