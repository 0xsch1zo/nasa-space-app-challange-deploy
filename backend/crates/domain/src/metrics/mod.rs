//! Health metrics: the extension point of the whole system.
//!
//! Every health parameter (blood pressure, bone density, ...) is described by
//! a declarative [`MetricDefinition`]. Validation, status assessment, body-map
//! aggregation, storage provisioning, the HTTP catalog and the frontend widgets
//! are all driven by these definitions, so no layer needs metric-specific code.

mod catalog;
mod definition;
pub mod definitions;
mod group;
mod reading;

pub use catalog::MetricCatalog;
pub use definition::*;
pub use group::MetricGroup;
pub use reading::*;
