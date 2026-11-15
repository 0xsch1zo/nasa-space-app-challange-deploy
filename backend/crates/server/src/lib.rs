//! Composition root: reads configuration and wires adapters into use cases.

use std::sync::Arc;

use adapter_auth::{Argon2Hasher, JwtTokenService};
use adapter_postgres::{
    PgCrewRepository, PgMeasurementRepository, PgPool, PgUserRepository, PgWellbeingRepository,
};
use application::ports::outbound::MeasurementRepository;
use application::services::{
    AlertService, AuthService, CrewService, HealthService, IngestionService, WellbeingService,
};
use domain::alerts::AlertEngine;
use domain::metrics::MetricCatalog;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
    pub bind_addr: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let var = |k: &str, default: &str| std::env::var(k).unwrap_or_else(|_| default.to_owned());
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("JWT_SECRET not set, using an insecure development secret");
            "dev-only-insecure-secret-change-me".to_owned()
        });
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            jwt_secret,
            jwt_ttl_minutes: var("JWT_TTL_MINUTES", "480").parse()?,
            bind_addr: var("BIND_ADDR", "0.0.0.0:8080"),
            cors_origins: var("CORS_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,tower_http=info".into()),
        )
        .init();
}

/// Everything the driving adapters need, built from concrete adapters.
pub struct Container {
    pub catalog: Arc<MetricCatalog>,
    pub auth: Arc<AuthService>,
    pub crew: Arc<CrewService>,
    pub health: Arc<HealthService>,
    pub alerts: Arc<AlertService>,
    pub wellbeing: Arc<WellbeingService>,
    pub ingestion: Arc<IngestionService>,
    pub hasher: Arc<Argon2Hasher>,
}

/// Runs migrations and provisions per-metric storage from the catalog.
pub async fn prepare_database(pool: &PgPool, catalog: &MetricCatalog) -> anyhow::Result<()> {
    adapter_postgres::migrate(pool).await?;
    let codes: Vec<_> = catalog
        .all()
        .iter()
        .map(|m| (m.code.clone(), m.scope))
        .collect();
    PgMeasurementRepository::new(pool.clone())
        .ensure_storage(&codes)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

pub fn build(pool: PgPool, config: &Config) -> Container {
    let catalog = Arc::new(MetricCatalog::builtin());
    let users = Arc::new(PgUserRepository::new(pool.clone()));
    let crew_repo = Arc::new(PgCrewRepository::new(pool.clone()));
    let measurements = Arc::new(PgMeasurementRepository::new(pool.clone()));
    let notes = Arc::new(PgWellbeingRepository::new(pool));
    let alert_engine = Arc::new(AlertEngine::standard());
    let hasher = Arc::new(Argon2Hasher);
    let tokens = Arc::new(JwtTokenService::new(
        config.jwt_secret.as_bytes(),
        chrono::Duration::minutes(config.jwt_ttl_minutes),
    ));

    Container {
        auth: Arc::new(AuthService::new(users, hasher.clone(), tokens)),
        crew: Arc::new(CrewService::new(
            crew_repo.clone(),
            measurements.clone(),
            catalog.clone(),
            alert_engine.clone(),
        )),
        alerts: Arc::new(AlertService::new(
            crew_repo.clone(),
            measurements.clone(),
            catalog.clone(),
            alert_engine,
        )),
        wellbeing: Arc::new(WellbeingService::new(crew_repo.clone(), notes)),
        health: Arc::new(HealthService::new(
            crew_repo.clone(),
            measurements.clone(),
            catalog.clone(),
        )),
        ingestion: Arc::new(IngestionService::new(
            crew_repo,
            measurements,
            catalog.clone(),
        )),
        hasher,
        catalog,
    }
}
