//! Driven ports: implemented by infrastructure adapters.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::crew::{Astronaut, AstronautId, Station, StationId, Subject, User};
use domain::metrics::{MetricCode, MetricScope, Reading, RecordedReading};
use domain::wellbeing::WellbeingNote;

use crate::error::{AppError, InfraError};
use crate::security::Principal;

pub type InfraResult<T> = Result<T, InfraError>;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> InfraResult<Option<User>>;
}

#[async_trait]
pub trait CrewRepository: Send + Sync {
    async fn find_astronaut(&self, id: AstronautId) -> InfraResult<Option<Astronaut>>;
    async fn list_astronauts(&self, station: StationId) -> InfraResult<Vec<Astronaut>>;
    async fn find_station(&self, id: StationId) -> InfraResult<Option<Station>>;
}

/// Time bucket used to downsample long series for charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bucket {
    Hour,
    Day,
    Week,
}

impl Bucket {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "hour" => Some(Bucket::Hour),
            "day" => Some(Bucket::Day),
            "week" => Some(Bucket::Week),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeRange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

/// Storage of the per-(subject, metric) history aggregates, where the
/// subject is an astronaut or (for environment metrics) a station.
///
/// Generic over metrics on purpose: the adapter must not need changes when a
/// metric is added.
#[async_trait]
pub trait MeasurementRepository: Send + Sync {
    /// Makes sure every metric has its storage (e.g. a table partition).
    /// Called at startup with the catalog's codes.
    async fn ensure_storage(&self, metrics: &[(MetricCode, MetricScope)]) -> InfraResult<()>;

    /// Appends validated readings and refreshes the "latest value" projection.
    async fn append(&self, readings: &[RecordedReading]) -> InfraResult<u64>;

    /// Latest reading of each metric of one subject.
    async fn latest(&self, subject: Subject) -> InfraResult<Vec<(MetricCode, Reading)>>;

    /// Latest reading of each astronaut-scoped metric for every astronaut on a station.
    async fn latest_for_station(
        &self,
        station: StationId,
    ) -> InfraResult<Vec<(AstronautId, MetricCode, Reading)>>;

    /// Ordered time series, optionally averaged into buckets.
    async fn series(
        &self,
        subject: Subject,
        metric: &MetricCode,
        range: TimeRange,
        bucket: Option<Bucket>,
    ) -> InfraResult<Vec<Reading>>;
}

/// Wellbeing notes are written by an external process; we only read them.
#[async_trait]
pub trait WellbeingRepository: Send + Sync {
    async fn latest_note(&self, astronaut: AstronautId) -> InfraResult<Option<WellbeingNote>>;
}

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, plain: &str) -> InfraResult<String>;
    fn verify(&self, plain: &str, hash: &str) -> bool;
}

#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub trait TokenService: Send + Sync {
    fn issue(&self, principal: &Principal) -> InfraResult<IssuedToken>;
    /// Must return `AppError::Unauthenticated` for invalid/expired tokens.
    fn verify(&self, token: &str) -> Result<Principal, AppError>;
}
