//! Writes reference data (stations, crew, accounts) for the demo seeder.
//! Health measurements are NOT written here: they go through the
//! `MeasurementIngestion` port like real device data would.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgSeedWriter {
    pool: PgPool,
}

impl PgSeedWriter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn has_users(&self) -> Result<bool, sqlx::Error> {
        let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(n > 0)
    }

    pub async fn reset(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "TRUNCATE wellbeing_notes, latest_station_measurements, station_measurements, \
             latest_measurements, measurements, users, astronauts, stations CASCADE",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn station(&self, id: Uuid, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO stations (id, name) VALUES ($1, $2)")
            .bind(id)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn astronaut(
        &self,
        id: Uuid,
        station: Uuid,
        full_name: &str,
        agency: &str,
        specialty: &str,
        mission_start: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO astronauts (id, station_id, full_name, agency, specialty, mission_start)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(station)
        .bind(full_name)
        .bind(agency)
        .bind(specialty)
        .bind(mission_start)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn user(
        &self,
        username: &str,
        display_name: &str,
        password_hash: &str,
        role: &str,
        station: Uuid,
        astronaut: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO users (id, username, display_name, password_hash, role, station_id, astronaut_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(display_name)
        .bind(password_hash)
        .bind(role)
        .bind(station)
        .bind(astronaut)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Stands in for the external wellbeing-note generator.
    pub async fn wellbeing_note(
        &self,
        astronaut: Uuid,
        written_at: DateTime<Utc>,
        body: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO wellbeing_notes (astronaut_id, written_at, body) VALUES ($1, $2, $3)",
        )
        .bind(astronaut)
        .bind(written_at)
        .bind(body)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
