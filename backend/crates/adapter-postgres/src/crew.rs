use application::ports::outbound::{CrewRepository, InfraResult};
use async_trait::async_trait;
use chrono::NaiveDate;
use domain::crew::{Astronaut, AstronautId, Station, StationId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::infra;

#[derive(Clone)]
pub struct PgCrewRepository {
    pool: PgPool,
}

impl PgCrewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

type AstronautRow = (Uuid, Uuid, String, String, String, NaiveDate);

fn to_astronaut((id, station, full_name, agency, specialty, start): AstronautRow) -> Astronaut {
    Astronaut {
        id: AstronautId(id),
        station_id: StationId(station),
        full_name,
        agency,
        specialty,
        mission_start: start,
    }
}

const ASTRONAUT_COLUMNS: &str = "id, station_id, full_name, agency, specialty, mission_start";

#[async_trait]
impl CrewRepository for PgCrewRepository {
    async fn find_astronaut(&self, id: AstronautId) -> InfraResult<Option<Astronaut>> {
        let row: Option<AstronautRow> = sqlx::query_as(&format!(
            "SELECT {ASTRONAUT_COLUMNS} FROM astronauts WHERE id = $1"
        ))
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;
        Ok(row.map(to_astronaut))
    }

    async fn list_astronauts(&self, station: StationId) -> InfraResult<Vec<Astronaut>> {
        let rows: Vec<AstronautRow> = sqlx::query_as(&format!(
            "SELECT {ASTRONAUT_COLUMNS} FROM astronauts WHERE station_id = $1 ORDER BY full_name"
        ))
        .bind(station.0)
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        Ok(rows.into_iter().map(to_astronaut).collect())
    }

    async fn find_station(&self, id: StationId) -> InfraResult<Option<Station>> {
        let row: Option<(Uuid, String)> =
            sqlx::query_as("SELECT id, name FROM stations WHERE id = $1")
                .bind(id.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
        Ok(row.map(|(id, name)| Station {
            id: StationId(id),
            name,
        }))
    }
}
