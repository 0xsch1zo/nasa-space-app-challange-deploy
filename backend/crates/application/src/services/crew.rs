use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::alerts::AlertEngine;
use domain::crew::{Astronaut, AstronautId};
use domain::metrics::{HealthStatus, MetricCatalog, MetricCode, Reading};

use super::{Authorizer, SnapshotReader};
use crate::error::{AppError, AppResult};
use crate::ports::inbound::{AlertCounts, CrewMemberSummary, CrewUseCase};
use crate::ports::outbound::{CrewRepository, MeasurementRepository};
use crate::security::{AccessPolicy, Principal};

pub struct CrewService {
    crew: Arc<dyn CrewRepository>,
    snapshots: SnapshotReader,
    catalog: Arc<MetricCatalog>,
    engine: Arc<AlertEngine>,
    authorizer: Authorizer,
}

impl CrewService {
    pub fn new(
        crew: Arc<dyn CrewRepository>,
        measurements: Arc<dyn MeasurementRepository>,
        catalog: Arc<MetricCatalog>,
        engine: Arc<AlertEngine>,
    ) -> Self {
        let authorizer = Authorizer::new(crew.clone());
        let snapshots = SnapshotReader::new(measurements);
        Self {
            crew,
            snapshots,
            catalog,
            engine,
            authorizer,
        }
    }

    fn summarize(
        &self,
        astronaut: Astronaut,
        station: domain::crew::Station,
        latest: &[(MetricCode, Reading)],
    ) -> CrewMemberSummary {
        let metric_statuses: Vec<_> = self
            .catalog
            .all()
            .iter()
            .map(|def| {
                let status = latest
                    .iter()
                    .find(|(code, _)| code == &def.code)
                    .map(|(_, r)| def.assess(r).status)
                    .unwrap_or(HealthStatus::Unknown);
                (def.code.clone(), status)
            })
            .collect();
        let overall_status = metric_statuses
            .iter()
            .map(|(_, s)| *s)
            .filter(|s| *s != HealthStatus::Unknown)
            .max()
            .unwrap_or(HealthStatus::Unknown);
        let last_measurement_at: Option<DateTime<Utc>> =
            latest.iter().map(|(_, r)| r.recorded_at).max();
        let now = Utc::now();
        let mission_day = (now.date_naive() - astronaut.mission_start).num_days() + 1;
        let alert_counts = AlertCounts::of(&self.engine.evaluate(&self.catalog, latest, now));
        CrewMemberSummary {
            astronaut,
            station,
            mission_day,
            overall_status,
            last_measurement_at,
            metric_statuses,
            alert_counts,
        }
    }
}

#[async_trait]
impl CrewUseCase for CrewService {
    async fn list_crew(&self, principal: &Principal) -> AppResult<Vec<CrewMemberSummary>> {
        if !AccessPolicy::can_list_crew(principal) {
            return Err(AppError::Forbidden);
        }
        // The caller's own station is missing: the token refers to data that
        // no longer exists (e.g. after a reset), so the session is stale.
        let station = self
            .crew
            .find_station(principal.station_id)
            .await?
            .ok_or(AppError::Unauthenticated)?;
        let astronauts = self.crew.list_astronauts(principal.station_id).await?;
        let snapshots = self.snapshots.for_station(station.id).await?;
        Ok(astronauts
            .into_iter()
            // Defence in depth: the repository is already station-scoped.
            .filter(|a| AccessPolicy::can_view(principal, a))
            .map(|a| {
                let latest = snapshots.for_astronaut(a.id);
                self.summarize(a, station.clone(), &latest)
            })
            .collect())
    }

    async fn get_astronaut(
        &self,
        principal: &Principal,
        id: AstronautId,
    ) -> AppResult<CrewMemberSummary> {
        let astronaut = self.authorizer.astronaut_visible_to(principal, id).await?;
        let station = self
            .crew
            .find_station(astronaut.station_id)
            .await?
            .ok_or(AppError::NotFound("station"))?;
        let latest = self.snapshots.for_astronaut(&astronaut).await?;
        Ok(self.summarize(astronaut, station, &latest))
    }
}
