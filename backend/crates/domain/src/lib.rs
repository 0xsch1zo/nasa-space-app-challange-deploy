//! Pure domain model of the astronaut health dashboard.
//!
//! No HTTP, SQL or crypto lives here. The central idea is that a health
//! parameter is *data* (a [`metrics::MetricDefinition`]) rather than code
//! paths scattered across layers, so adding one is a single new file in
//! [`metrics::definitions`] plus one line in [`metrics::MetricCatalog::builtin`].

pub mod alerts;
pub mod body;
pub mod crew;
pub mod error;
pub mod metrics;
pub mod wellbeing;

pub use error::DomainError;
