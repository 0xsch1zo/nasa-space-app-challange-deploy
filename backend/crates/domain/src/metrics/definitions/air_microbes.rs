use crate::metrics::{Bounds, ChartKind, MetricDefinition, MetricScope};

use super::reading;

/// Airborne microbes classified as potentially harmful (air sampler, daily).
/// ISS limit for airborne bacteria is 1000 CFU/m³.
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("air_microbes", "Jakość powietrza: mikroby")
        .description("Liczba mikrobów w powietrzu uznanych za potencjalnie szkodliwe.")
        .chart(ChartKind::Line)
        .group("environment")
        .scope(MetricScope::Station)
        .sampling_hours(24)
        .component(reading(
            "harmful_cfu",
            "Mikroby potencjalnie szkodliwe",
            "CFU/m³",
            Bounds::between(0.0, 100_000.0),
            Bounds::at_most(300.0),
            Bounds::at_most(1000.0),
            0,
            140.0,
            30.0,
            0.0,
        ))
        .build()
}
