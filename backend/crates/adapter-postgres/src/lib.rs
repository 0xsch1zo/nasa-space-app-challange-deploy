//! PostgreSQL adapters for the outbound repository ports.

mod crew;
mod measurements;
mod seed;
mod users;
mod wellbeing;

pub use crew::PgCrewRepository;
pub use measurements::PgMeasurementRepository;
pub use seed::PgSeedWriter;
pub use users::PgUserRepository;
pub use wellbeing::PgWellbeingRepository;

use application::error::InfraError;
use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub(crate) fn infra(e: sqlx::Error) -> InfraError {
    tracing::error!(error = %e, "database error");
    InfraError::new(e)
}
