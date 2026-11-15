use std::collections::HashMap;
use std::sync::Arc;

use domain::crew::{Astronaut, AstronautId, StationId, Subject};
use domain::metrics::{MetricCode, Reading};

use crate::error::AppResult;
use crate::ports::outbound::MeasurementRepository;

pub type Latest = Vec<(MetricCode, Reading)>;

/// Loads the "latest value" projection for a profile: the astronaut's own
/// metrics plus the environment metrics of their station.
#[derive(Clone)]
pub struct SnapshotReader {
    measurements: Arc<dyn MeasurementRepository>,
}

/// Latest values of every astronaut on a station, loaded in two queries.
pub struct StationSnapshots {
    own: HashMap<AstronautId, Latest>,
    environment: Latest,
}

impl StationSnapshots {
    pub fn for_astronaut(&self, id: AstronautId) -> Latest {
        let mut latest = self.own.get(&id).cloned().unwrap_or_default();
        latest.extend(self.environment.iter().cloned());
        latest
    }
}

impl SnapshotReader {
    pub fn new(measurements: Arc<dyn MeasurementRepository>) -> Self {
        Self { measurements }
    }

    pub async fn for_astronaut(&self, astronaut: &Astronaut) -> AppResult<Latest> {
        let mut latest = self
            .measurements
            .latest(Subject::Astronaut(astronaut.id))
            .await?;
        latest.extend(
            self.measurements
                .latest(Subject::Station(astronaut.station_id))
                .await?,
        );
        Ok(latest)
    }

    pub async fn for_station(&self, station: StationId) -> AppResult<StationSnapshots> {
        let environment = self.measurements.latest(Subject::Station(station)).await?;
        let mut own: HashMap<AstronautId, Latest> = HashMap::new();
        for (id, code, reading) in self.measurements.latest_for_station(station).await? {
            own.entry(id).or_default().push((code, reading));
        }
        Ok(StationSnapshots { own, environment })
    }
}
