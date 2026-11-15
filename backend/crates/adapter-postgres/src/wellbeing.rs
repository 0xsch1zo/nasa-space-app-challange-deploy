use application::ports::outbound::{InfraResult, WellbeingRepository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::crew::AstronautId;
use domain::wellbeing::WellbeingNote;
use sqlx::PgPool;

use crate::infra;

#[derive(Clone)]
pub struct PgWellbeingRepository {
    pool: PgPool,
}

impl PgWellbeingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WellbeingRepository for PgWellbeingRepository {
    async fn latest_note(&self, astronaut: AstronautId) -> InfraResult<Option<WellbeingNote>> {
        let row: Option<(DateTime<Utc>, String)> = sqlx::query_as(
            "SELECT written_at, body FROM wellbeing_notes
             WHERE astronaut_id = $1 ORDER BY written_at DESC LIMIT 1",
        )
        .bind(astronaut.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;
        Ok(row.map(|(written_at, body)| WellbeingNote {
            astronaut_id: astronaut,
            written_at,
            body,
        }))
    }
}
