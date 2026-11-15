use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use domain::alerts::{Alert, AlertEngine};
use domain::crew::AstronautId;
use domain::metrics::MetricCatalog;

use super::{Authorizer, SnapshotReader};
use crate::error::AppResult;
use crate::ports::inbound::AlertUseCase;
use crate::ports::outbound::{CrewRepository, MeasurementRepository};
use crate::security::Principal;

pub struct AlertService {
    authorizer: Authorizer,
    snapshots: SnapshotReader,
    catalog: Arc<MetricCatalog>,
    engine: Arc<AlertEngine>,
}

impl AlertService {
    pub fn new(
        crew: Arc<dyn CrewRepository>,
        measurements: Arc<dyn MeasurementRepository>,
        catalog: Arc<MetricCatalog>,
        engine: Arc<AlertEngine>,
    ) -> Self {
        Self {
            authorizer: Authorizer::new(crew),
            snapshots: SnapshotReader::new(measurements),
            catalog,
            engine,
        }
    }
}

#[async_trait]
impl AlertUseCase for AlertService {
    async fn alerts(&self, principal: &Principal, id: AstronautId) -> AppResult<Vec<Alert>> {
        let astronaut = self.authorizer.astronaut_visible_to(principal, id).await?;
        let latest = self.snapshots.for_astronaut(&astronaut).await?;
        Ok(self.engine.evaluate(&self.catalog, &latest, Utc::now()))
    }
}
